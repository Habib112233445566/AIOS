# Running AIOS with Graphical Desktop in Google Colab

This directory contains `AIOS_Kali_Desktop_Colab.ipynb`, a ready-to-run Jupyter notebook designed to launch a full graphical Linux desktop (XFCE4 + noVNC + Cloudflare Tunnel) with Kali Linux security tools, Rust binaries, and Ollama AI models.

## How to Run:

1. Go to [Google Colab](https://colab.research.google.com).
2. Click **File** ➔ **Upload notebook**.
3. Select `AIOS_Kali_Desktop_Colab.ipynb` from this folder (or drag and drop it into Colab).
4. In the Colab top menu, set the accelerator:
   - **Runtime** ➔ **Change runtime type** ➔ Select **T4 GPU** ➔ Click **Save**.
5. Run the cells in order from top to bottom (click the **Play ▶** button on each cell).
6. When **Step 4** finishes, it will print a link that looks like:
   ```
   🖥️ CLICK THIS LINK TO ACCESS YOUR GRAPHICAL DESKTOP:
      https://random-words.trycloudflare.com/vnc.html?autoconnect=true&resize=scale
      VNC Password: aios1234
   ```
7. Click the link! A new browser tab will open displaying your full Linux graphical desktop.
8. Inside the desktop, you can open the terminal, launch Kali tools, and execute `aiosh` commands.
