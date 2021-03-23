wasm_bindgen("pkg/svg_web_bg.wasm").catch(console.error)
.then(init);

let utf8decoder = new TextDecoder();
let utf8encoder = new TextEncoder();

function show_logo() {
    fetch("logo.svg")
    .then(r => r.arrayBuffer())
    .then(function(buf) {
        document.getElementById("svg-input").value = utf8decoder.decode(buf);
        show_data(new Uint8Array(buf));
    });
}

let FONTS = null;
async function init() {
    FONTS = await load_fonts();
    show_logo();
}

async function load_fonts() {
    const fonts = [
        "resources/latinmodern-math.otf",
        "resources/NotoNaskhArabic-Regular.ttf",
        "resources/NotoSerifBengali-Regular.ttf",
    ];
    let fonts_data = await Promise.all(
        fonts.map(url => fetch(url).then(r => r.arrayBuffer()))
    );
    let builder = new wasm_bindgen.FontBuilder();
    fonts_data.forEach(data => builder.add(data));
    return builder.build();
}

function set_scroll_factors() {}

function drop_handler(e) {
    e.stopPropagation();
    e.preventDefault();
    show(e.dataTransfer.files[0]);
}
function dragover_handler(e) {
    e.stopPropagation();
    e.preventDefault();
}

function display(msg) {
}


let view;
function init_view(data) {
    let canvas = document.getElementById("canvas");
    view = wasm_bindgen.show(canvas, data);

    let requested = false;
    function animation_frame(time) {
        requested = false;
        view.animation_frame(time);
    }
    function check(request_redraw) {
        if (request_redraw && !requested) {
            window.requestAnimationFrame(animation_frame);
            requested = true;
        }
    }

    window.addEventListener("keydown", e => check(view.key_down(e)), {capture: true});
    window.addEventListener("keyup", e => check(view.key_up(e)), {capture: true});
    canvas.addEventListener("mousemove", e => check(view.mouse_move(e)));
    canvas.addEventListener("mouseup", e => check(view.mouse_up(e)));
    canvas.addEventListener("mousedown", e => check(view.mouse_down(e)));
    window.addEventListener("resize", e => check(view.resize(e)));
    view.render();
}

function view_area_input(event) {
    let data = utf8encoder.encode(event.target.value);
    show_data(data);
}

function show_data(data) {
    try {
        init_view(data);
    } catch (e) {
        display("oops. try another one.");
    }
}

function show(file) {
    let reader = new FileReader();
    reader.onload = function() {
        let data = new Uint8Array(reader.result);
        show_data(data);
    };
    reader.readAsArrayBuffer(file);
}

function open() {
    var input = document.createElement('input');
    input.type = 'file';
    input.onchange = e => { 
        // getting a hold of the file reference
        var file = e.target.files[0]; 
        show(file);
    };
    input.click();
}

document.addEventListener("drop", drop_handler, false);
document.addEventListener("dragover", dragover_handler, false);
