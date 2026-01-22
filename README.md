# yesser-todo-cli
a CLI app for managing your tasks

# Installation
## Cargo
If you have Cargo installed (on any platform), you can install with:
```bash
cargo install yesser-todo-cli
```

## Windows
On Windows, you can use the MSI installer in the releases page.
You can also use the Scoop from the yesser-studios bucket:
```pwsh
scoop bucket add yesser-studios https://github.com/yesser-studios/scoop-bucket
scoop install yesser-todo-cli
```

## MacOS
You can use the yesser-studios Homebrew tap:
```bash
brew tap yesser-studios/tap
brew install yesser-todo-cli
```

## Linux
On Fedora 41-43 or Rawhide, you can use the yesser-studios COPR:
```bash
sudo dnf copr enable yesseruser/yesser-studios
sudo dnf install yesser-todo-cli
```
On other distributions, you will need to use cargo or build the project from source.

# Publishing (for maintainers)
## Crates.io
*Note: Make sure you're logged in with `cargo login`*

First, run `cargo package` to generate a `.crate` file. 
Then, run `cargo publish` to upload the crate to crates.io.

## Homebrew
First, create a GitHub release to make a tag for Homebrew to download. 
Then, download the source code as a `.tar.gz` file, generate a sha256 hash, and add the hash and link to the formula.
Next, on both an ARM64 macOS machine run `brew install --build-bottle --bottle-arch=arm64_sonoma yesser-todo-cli`.
On a x64 Linux machine run `brew install --build-bottle --bottle-arch=x86_64_linux yesser-todo-cli`.
On both machines, run `brew bottle yesser-todo-cli` and upload the file to GitHub releases. 
Generate a sha256 hash for the file and replace the corresponding field in the formula.
Make sure to change the `root_url` as well. Also, change the version in the test's assertion.

## Windows build
On a x64 Windows machine, install cargo-wix with `cargo install cargo-wix` and run `cargo wix`. 
Next, upload the generated `.msi` file to the GitHub release.

## Fedora COPR build
1. On a Fedora machine with rpm build tools and rust2rpm installed, run the update.sh script located at https://github.com/yesser-studios/rpms/tree/main/yesser-todo-cli.  
2. Commit and push the changes. COPR should build the new commit automatically.

# Server
> [!CAUTION]
> **Server functionality is not yet production-ready.**  
> Accessing the server multiple times at the same time *will* cause race conditions and potentially edit incorrect tasks and/or crash the server.
## Usage guide:
Run the `yesser-todo-server` crate. This will open port 6982 and listen for HTTP traffic.
## Endpoints
- `GET /tasks` returns a JSON containing an array of `Task` objects, such as: `[{name: "example", done: true}]`
- `POST /add` accepts a body JSON representation of a string, such as: `"example"`.
This string will be used as the name for a new task. Returns a JSON of the generated `Task` object.
- `DELETE /remove` accepts a body JSON representation of an integer, such as: `5`.
The task with the index of the given integer will be deleted. The index can be queried with `GET /index` (see below)
- `POST /done` accepts a body JSON representation of an integer, such as: `5`.
The task with the given index will be marked as done. Returns a JSON of the modified `Task` object.
- `POST /undone` accepts a body JSON representation of an integer, such as: `5`.
The task with the given index will be marked as undone. Returns a JSON of the modified `Task` object.
- `DELETE /clear` will delete all tasks.
- `DELETE /cleardone` will delete all tasks marked as done.
- `GET /index` accepts a body JSON representation of a string.
It will return the index at which the given string first appears.
