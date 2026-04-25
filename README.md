### In LINUX avoid Flatpack for REAPER installation.

### Add versioned external C libs

```bash
git submodule add -b main https://github.com/free-audio/clap external/clap-1.2.7
cd external/clap-1.2.7
git checkout tags/1.2.7

git submodule add -b main https://github.com/sdatkinson/NeuralAmpModelerCore.git external/neural-amp-modeler-0.5.1
cd external/neural-amp-modeler-0.5.1
git checkout tags/v0.5.1

git submodule add -b release-3.4.x https://github.com/libsdl-org/SDL external/sdl-3.4.4
cd external/sdl-3.4.4
git checkout tags/release-3.4.4
```

### Compile

```bash
# Compilar
just build

# Output (macOS: bundle, Linux/Windows: archivo plano)
# -> build/Release/clap-gain.clap/        (macOS bundle)
# -> build/Release/clap-gain.clap         (Linux/Windows)
# -> build/Release/libSDL3.dylib|.so|.dll (junto al plugin)
```

### Validate plugin

```bash
clap-validator validate build/Release/nam-player.clap
clap-info build/Release/nam-player.clap
```
