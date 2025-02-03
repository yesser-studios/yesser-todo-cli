# yesser-todo-cli
a CLI app for managing your tasks

# Publishing (for developers)
## Crates.io
*Note: Make sure you're logged in with `cargo login`*

First, run `cargo package` to generate a `.crate` file. 
Then, run `cargo publish` to upload the crate to crates.io.

## Homebrew
First, create a GitHub release to make a tag for Homebrew to download. 
Then, download the source code as a `.tar.gz` file, generate a sha256 hash, and add the hash and link to the formula.
Next, on both an ARM64 MacOS machine run `brew install --build-bottle --arch=arm64_sonoma yesser-todo-cli`.
On a x64 Linux machine run `brew install --build-bottle --arch=x86_64_linux yesser-todo-cli`.
On both machines, run `brew bottle yesser-todo-cli` and upload the file to github releases. 
Generate a sha256 hash for the file and replace the corresponding field in the formula.
Make sure to change the `root_url` as well. Also, change the version in the test's assertion.

## Windows build
On an x64 Windows machine, install cargo-wix with `cargo install cargo-wix` and run `cargo wix`. 
Next, upload the generated `.msi` file to the GitHub release.