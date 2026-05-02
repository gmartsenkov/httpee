# vhs script: generate docs/assets/demo.gif
# Run with: vhs demo.tape

Output docs/assets/demo.gif

Set Shell "zsh"
Set FontFamily "JetBrainsMono Nerd Font Mono"
Set FontSize 22
Set Width 1300
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 60ms

# Make ./target/release available as `httpee` and clear the prompt
Hide
Type "export PATH=$PWD/target/release:$PATH && clear"
Enter
Sleep 400ms
Show

Type "httpee"
Sleep 500ms
Enter
Sleep 2200ms

Type "clear"
Enter
Sleep 200ms

Type "httpee --verbose example/ping"
Sleep 500ms
Enter
Sleep 3000ms

Type "clear"
Enter
Sleep 200ms

Type "httpee example/users/sho"
Sleep 500ms
Enter
Sleep 2000ms
