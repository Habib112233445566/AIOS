#!/usr/bin/env python3
"""AIOS-Kernel-SLM Fast Unsloth Training & GGUF Quantization Script.

Fine-tunes Qwen2.5-1.5B (or 0.5B) using Unsloth on Google Colab / GPU.
Exports quantized GGUF (Q4_K_M) ready for consumer CPU execution via Ollama / llama.cpp.
"""

import os
import json
import torch
from pathlib import Path
from datasets import Dataset

BASE_DIR = Path(__file__).resolve().parent
DATA_PATH = BASE_DIR / "data" / "aios_train.jsonl"
OUTPUT_GGUF_NAME = "aios-kernel-1.5b-q4_k_m"


def format_chatml(messages):
    text = ""
    for m in messages:
        text += f"<|im_start|>{m['role']}\n"
        if m.get("content"):
            text += f"{m['content']}\n"
        for tc in m.get("tool_calls", []):
            fn = tc.get("function", {})
            name = fn.get("name", "")
            args = fn.get("arguments", "{}")
            text += f"<tool_call>\n{{\"name\": \"{name}\", \"arguments\": {args}}}\n</tool_call>\n"
        text += "<|im_end|>\n"
    return text


def main():
    print("=" * 65)
    print("       AIOS-Kernel-SLM Unsloth Fine-Tuning & GGUF Export")
    print("=" * 65)

    # 1. Verify GPU
    if not torch.cuda.is_available():
        print("[-] Error: CUDA GPU not detected! Make sure Colab runtime is set to GPU (T4/A100).")
        return

    gpu_name = torch.cuda.get_device_name(0)
    print(f"[+] Active GPU: {gpu_name}")

    try:
        from unsloth import FastLanguageModel
        from trl import SFTTrainer
        from transformers import TrainingArguments
    except ImportError as e:
        print(f"[-] Missing library: {e}")
        print("    Run: pip install \"unsloth[colab-new] @ git+https://github.com/unslothai/unsloth.git\"")
        print("         pip install --no-deps xformers trl peft accelerate bitsandbytes datasets")
        return

    # 2. Load Model in 4-bit
    max_seq_length = 2048
    model_id = "Qwen/Qwen2.5-1.5B-Instruct"
    print(f"[*] Loading base model: {model_id} (4-bit QLoRA)...")

    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=model_id,
        max_seq_length=max_seq_length,
        load_in_4bit=True,
    )

    # 3. Add LoRA adapters
    print("[*] Adding LoRA adapters to projection layers...")
    model = FastLanguageModel.get_peft_model(
        model,
        r=16,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"],
        lora_alpha=32,
        lora_dropout=0,
        bias="none",
        use_gradient_checkpointing="unsloth",
        random_state=42,
    )

    # 4. Load & serialize dataset
    print(f"[*] Loading dataset from: {DATA_PATH}...")
    if not DATA_PATH.exists():
        print(f"[-] Error: Dataset not found at {DATA_PATH}")
        return

    samples = []
    with open(DATA_PATH, "r", encoding="utf-8") as f:
        for line in f:
            data = json.loads(line)
            samples.append({"text": format_chatml(data["messages"])})

    dataset = Dataset.from_list(samples)
    print(f"[+] Loaded {len(dataset)} verified dialogues.")

    # 5. Fine-Tuning Execution
    print("[*] Starting SFT training (300 steps, ~15 mins on T4 GPU)...")
    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        dataset_text_field="text",
        max_seq_length=max_seq_length,
        dataset_num_proc=2,
        packing=False,
        args=TrainingArguments(
            per_device_train_batch_size=4,
            gradient_accumulation_steps=4,
            warmup_steps=20,
            max_steps=300,
            learning_rate=2e-4,
            fp16=not torch.cuda.is_bf16_supported(),
            bf16=torch.cuda.is_bf16_supported(),
            logging_steps=10,
            optim="adamw_8bit",
            weight_decay=0.01,
            lr_scheduler_type="linear",
            seed=42,
            output_dir="checkpoints",
            report_to="none",
        ),
    )

    trainer.train()
    print("[+] Model fine-tuning completed successfully!")

    # 6. Quantize and export to GGUF
    print(f"[*] Exporting model directly to GGUF ({OUTPUT_GGUF_NAME})...")
    model.save_pretrained_gguf(OUTPUT_GGUF_NAME, tokenizer, quantization_method="q4_k_m")
    print(f"[+] GGUF exported: {OUTPUT_GGUF_NAME}.gguf is ready in {BASE_DIR}!")


if __name__ == "__main__":
    main()
