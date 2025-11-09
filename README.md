<img src="assets/run-animals-on-the-loose.gif" width="300" alt="Run Animals on the Loose">

# Hunt

Track down and eliminate unused translation keys with the precision of a predator! 🦁

## Install

```bash
git clone https://github.com/JeevesInc/hunt.git
cd hunt
chmod +x install.sh
./install.sh
```

or build manually:

```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/hunt ~/.local/bin/hunt
```

Then add `~/.local/bin` to your PATH. Add this line to `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell: `source ~/.bashrc` or `source ~/.zshrc`

## Usage

**Check for unused keys:**

```bash
hunt public/locales/en-US/
```

**Automatically remove unused keys:**

```bash
hunt public/locales/en-US/ --clear
```

<img src="assets/demo.gif" width="500" alt="Demo">

<img src="assets/demo-screenshot.png" width="500" alt="Demo Screenshot">