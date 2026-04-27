```bash
git submodule add -b main https://github.com/free-audio/clap external/clap-1.2.7
cd external/clap-1.2.7
git checkout tags/1.2.7

git submodule add -b main https://github.com/sdatkinson/NeuralAmpModelerCore.git external/neural-amp-modeler-0.5.1
cd external/neural-amp-modeler-0.5.1
git checkout tags/v0.5.1

git submodule update --init --recursive
```

### Dev

```bash
watchexec -r -e html,css,rs -- just dev
```

### Build

```bash
just build
```

### Validate plugin

```bash
clap-validator validate X
clap-info X
```
