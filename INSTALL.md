# Install Math Practice

After [setting up PrimitiveOS](https://github.com/PrimitiveOS/kindle-primitive-launcher/blob/main/docs/INSTALL.md#first-time-setup-over-usb), open **Store** from Launcher.
Find **Math Practice** under **Apps**, tap **Get**, then open it from the Launcher grid.

## Build a local package

Run from this checkout with Tools, Launcher, and Build next to it.

```sh
cargo zigbuild --offline --locked --release -p math-practice-kindle --target armv7-unknown-linux-musleabihf
sh scripts/stage.sh armhf
python3 ../doannt-kindle-build/scripts/package.py --id primitive-math-practice --dir math-practice \
  --version 0.1.0 --arch armhf --stage build/stage --license LICENSE \
  --docs-license LICENSE-DOCS --install INSTALL.md --out dist
```
