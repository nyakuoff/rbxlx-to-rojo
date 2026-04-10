# rbxl2rojo (supports .rbxl, .rbxm, .rbxlx, .rbxmx)
Tool to convert existing Roblox games into Rojo projects by reading their `rbxl`, `rbxm`, `rbxlx`, or `rbxmx` files.

# Using rbxl2rojo
## Setup
Before you can use rbxl2rojo, you need the following:

- At least Rojo 0.5.0 Alpha 12 or higher to use the tool.
- A place/model file (`.rbxl`, `.rbxm`, `.rbxlx`, or `.rbxmx`) that has scripts

If there aren't any scripts in the selected file, rbxl2rojo will return an error.

Download the latest release of rbxl2rojo here: https://github.com/nyakuoff/rbxl2rojo/releases
## Porting the game
Before you can port your game into Rojo projects, you need a place/model file. If you have an existing game that isn't exported:

- Go to studio, click on any place, and then click on File -> Save to file as.

- Create a folder and name it whatever you want.
### Steps to port the game:
1. Open a terminal in the folder where `rbxl2rojo` is installed.
2. Run one of these commands:
	- `rbxl2rojo` (opens file/folder picker dialogs)
	- `rbxl2rojo /path/to/place.rbxmx /path/to/output-folder`
3. Wait for conversion to finish, then open the generated project folder.

If you followed the steps correctly, you should see something that looks like this:
![](assets/folders.png)

Congratulations, you successfully ported an existing game using rbxl2rojo!

## License
rbxl2rojo is available under The Mozilla Public License, Version 2. Details are available in [LICENSE.md](LICENSE.md).
