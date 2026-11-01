# NGLB Dictate

NGLB Dictate is NEUGELB's managed build of the open-source speech-to-text app
[Handy](https://github.com/cjpais/handy) by CJ Pais, used under the MIT License
(see `LICENSE`, Copyright (c) 2025 CJ Pais).

The Handy name, logo and icon are not part of the MIT grant and are not used here.
This build is unofficial, is not endorsed by or affiliated with the Handy project,
and contains NEUGELB modifications (local-only policy layer, no network model
downloads, no updater, retention limits). See `NGLB-PATCHES.md`.

## Bundled models

| Model | Licence | Source |
|-------|---------|--------|
| Parakeet Ultra (NVIDIA Parakeet family, GGUF quantisation Q8_0 by the Handy project; converted/quantised, not modified otherwise) | CC BY 4.0 | https://huggingface.co/handy-computer/parakeet-ultra-gguf (revision 39eeb55181f0d354fd934f06e92fd8d5037fed8e) |
| VoxLingua107 ECAPA language identification (GGUF F16) | Apache-2.0 | https://huggingface.co/handy-computer/lang-id-voxlingua107-ecapa-gguf (revision 41cf26039e317c6b8883bf0a61b0785dc663fc26) |
