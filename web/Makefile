DST = /home/sebk/data/svg_view

build:
	wasm-pack build -t no-modules --release

publish:
	cp pkg/svg_web.js pkg/svg_web_bg.wasm $(DST)/pkg/
	cp index.html style.css index.js $(DST)/
	git -C $(DST) commit -a -m "update"
	git -C $(DST) push

.PHONY:	all

all:	build
