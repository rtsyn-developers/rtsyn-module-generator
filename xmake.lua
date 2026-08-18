local project_name = "rtsyn-module-generator"

set_license("GPL-3.0-or-later")

add_rules("mode.debug", "mode.release")
add_rules("plugin.compile_commands.autoupdate")
set_defaultmode("release")

target(project_name)
set_kind("binary")
add_files("src/main.rs")

before_build(function(target)
	os.execv("git", {
		"submodule",
		"update",
		"--init",
		"--recursive",
		"--force",
	}, {
		curdir = os.projectdir(),
	})
end)

add_tests("generate-adder", {
	runargs = {
		"--force",
		path.join(os.projectdir(), "examples/adder.toml"),
		path.join(os.tmpdir(), "rtsyn-module-plugin-adder"),
	},
})

add_tests("generate-device", {
	runargs = {
		"--force",
		path.join(os.projectdir(), "examples/device.toml"),
		path.join(os.tmpdir(), "rtsyn-module-generator-device"),
	},
})
