# pixelsmith
### (working title)

![License](https://img.shields.io/crates/l/imgui-wgpu)

My entry for the 2022 [Tool Jam 2](https://itch.io/jam/the-tool-jam-2) on [itch.io](https://itch.io).

For ease of development, the following repos have been vendored:

* imgui-rs/imgui-rs (Apache 2.0 / MIT dual license)
* Yatekii/imgui-wgpu-rs (Apache 2.0 / MIT dual license)
* trashbyte/toolbelt (MIT license)

For more info, see `vendored.txt`. To avoid having to deal with submodules or recursive git jank from nested repos, the vendored repos are present sans `.git[hub]` folders. The first commit to this repo has the repos unmodified aside from this removal, so `git diff $(git mktree </dev/null) HEAD` (bash) or `git diff $($null | git mktree) HEAD` (powershell) should show all of the changes I made to them.
