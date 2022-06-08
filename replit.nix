{ pkgs }: {
	deps = [
		pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
		pkgs.cargo-edit
		pkgs.rust-analyzer
		pkgs.rustup
		pkgs.vim
		pkgs.killall
		pkgs.htop
		pkgs.python310
	];
}
