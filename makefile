build:
	cargo build
	cp ./target/debug/libgd_ocean.so /home/zmanjaro/yarhar/yarhar/addons/gd_ocean/debug/libgd_ocean.so
build-release:
	cargo build --release
	cp ./target/release/libgd_ocean.so /home/zmanjaro/yarhar/yarhar/addons/gd_ocean/release/libgd_ocean.so
# build-all:

