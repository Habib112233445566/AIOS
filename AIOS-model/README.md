# AIOS-model Workspace

This directory is the dedicated, isolated workspace for building, dataset synthesis, fine-tuning, and packaging the custom **`AIOS-Kernel-SLM`** (Small Language Model).

> [!IMPORTANT]
> **Boundary Rule for Model Training AI Agent:**
> - All training scripts, dataset generators, synthetic datasets, checkpoints, adapters, GGUF models, and Modelfiles must be kept strictly inside this `AIOS-model/` directory.
> - Do not modify or edit any files outside of `AIOS-model/` (e.g. `code/aiosh-rust/`, `ai_agent.py`, `docs/`, `tools/`).
> - For full technical roadmap and specifications, see [`../AIOS_CUSTOM_MODEL_MASTER_GUIDE.md`](../AIOS_CUSTOM_MODEL_MASTER_GUIDE.md).

## Recommended Structure
```
AIOS-model/
├── README.md
├── generate_dataset.py    # Synthetic dataset generator
├── train_slm.py           # Fine-tuning script (Unsloth / TRL)
├── Modelfile              # Ollama modelfile
├── data/                  # Generated JSONL datasets
│   └── aios_train.jsonl
├── checkpoints/           # Training checkpoints & LoRA adapters
└── merged/                # Merged GGUF models
```
