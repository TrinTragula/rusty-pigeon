import("rustypigeonwasm").then(function (rusty) {
    onmessage = function (e) {
        if (e.data.name == "set_pos") {
            rusty.set_pos(e.data.argument);
        }
        if (e.data.name == "get_move") {
            let move = rusty.get_move(e.data.argument);
            postMessage({ name: "move", argument: move });
        }
    }
});