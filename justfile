plugin_name := "neural_amp_modeler"
lib_name    := "neural_amp_modeler"

build_dir   := "target/release"

styles:
    npx @tailwindcss/cli -i ./src/gui/app/style/index.css -o ./src/gui/app/style/output.css -m

build:
    just styles
    cargo build --release
    just bundle

[linux]
bundle:
    clap-validator validate {{build_dir}}/lib{{lib_name}}.so
    cp {{build_dir}}/lib{{lib_name}}.so {{build_dir}}/{{plugin_name}}.clap
    yes | cp -r {{build_dir}}/{{plugin_name}}.clap ~/.clap/

[windows]
bundle:
    clap-validator validate {{build_dir}}\{{lib_name}}.dll
    copy {{build_dir}}\{{lib_name}}.dll {{build_dir}}\{{plugin_name}}.clap

[macos]
bundle:
    mkdir -p {{build_dir}}/{{plugin_name}}.clap/Contents/MacOS
    cp {{build_dir}}/lib{{lib_name}}.dylib {{build_dir}}/{{plugin_name}}.clap/Contents/MacOS/{{plugin_name}}
    just _plist
    mkdir -p ~/Library/Audio/Plug-Ins/CLAP
    ditto {{build_dir}}/{{plugin_name}}.clap ~/Library/Audio/Plug-Ins/CLAP/{{plugin_name}}.clap
    codesign --force --deep --sign - ~/Library/Audio/Plug-Ins/CLAP/{{plugin_name}}.clap
    clap-validator validate ~/Library/Audio/Plug-Ins/CLAP/{{plugin_name}}.clap

[macos]
_plist:
    #!/usr/bin/env sh
    cat > {{build_dir}}/{{plugin_name}}.clap/Contents/Info.plist << 'EOF'
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>CFBundleExecutable</key>
        <string>neural_amp_modeler</string>
        <key>CFBundleIdentifier</key>
        <string>com.enguidanosweb.NeuralAmpModeler</string>
        <key>CFBundleVersion</key>
        <string>0.0.1</string>
        <key>CFBundlePackageType</key>
        <string>BNDL</string>
        <key>CFBundleSignature</key>
        <string>????</string>
    </dict>
    </plist>
    EOF
