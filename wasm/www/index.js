import Worker from "worker-loader!./worker.js";
import { Chessground } from 'chessground';
import { Chess, SQUARES } from 'chess.js';

let game = new Chess();
const board = Chessground(document.getElementById("chessboard"), {
    events: {
        move: (orig, dest, capturedPiece) => {
            console.log(orig, dest, capturedPiece);
        }
    },
    movable: {
        color: 'white',
        free: false,
        dests: toDests(game),
    },
    draggable: {
        showGhost: true
    }
});
board.set({
    movable: { events: { after: aiPlay(board, game) } }
})

resizeBoard();

window.addEventListener('resize', function (event) {
    resizeBoard();
}, true);

function resizeBoard() {
    const chessboard = document.getElementById("chessboard");
    const size = Math.min(document.body.offsetWidth - 15, 480) + "px";
    chessboard.style.width = size;
    chessboard.style.height = size;
}

const rustyWorker = new Worker();
rustyWorker.onmessage = function (e) {
    if (e.data.name == "move") {
        let move = e.data.argument;
        move = game.move(move, { sloppy: true });
        board.move(move.from, move.to);
        board.set({
            turnColor: toColor(game),
            movable: {
                color: toColor(game),
                dests: toDests(game)
            }
        });
        rustyWorker.postMessage({ name: "set_pos", argument: game.fen() });
        document.getElementById("status").innerHTML = "Your turn!";
        document.getElementById("playAsWhite").removeAttribute("disabled");
        document.getElementById("playAsBlack").removeAttribute("disabled");
    }
}

function makeRustyMove() {
    rustyWorker.postMessage({ name: "set_pos", argument: game.fen() });
    document.getElementById("status").innerHTML = "Thinking...";
    document.getElementById("playAsWhite").setAttribute("disabled", "disabled");
    document.getElementById("playAsBlack").setAttribute("disabled", "disabled");
    rustyWorker.postMessage({ name: "get_move", argument: document.getElementById("movetime").value * 1000 });
}

function toDests(chess) {
    const dests = new Map();
    SQUARES.forEach(s => {
        const ms = chess.moves({ square: s, verbose: true });
        if (ms.length) dests.set(s, ms.map(m => m.to));
    });
    return dests;
}

function toColor(chess) {
    return (chess.turn() === 'w') ? 'white' : 'black';
}

function aiPlay(cg, chess) {
    return (orig, dest) => {
        chess.move({ from: orig, to: dest });
        makeRustyMove()
    };
}

document.getElementById("playAsWhite").onclick = () => resetPos(false);
document.getElementById("playAsBlack").onclick = () => resetPos(true);
function resetPos(playAsBlack) {
    document.getElementById("status").innerHTML = "Resetting...";
    game.reset();
    rustyWorker.postMessage({ name: "set_pos", argument: game.fen() });
    board.set({
        turnColor: toColor(game),
        movable: {
            color: toColor(game),
            dests: toDests(game)
        },
        fen: game.fen(),
        lastMove: null

    })
    document.getElementById("status").innerHTML = "Let's play!";
    if (playAsBlack) {
        board.set({
            orientation: "black"
        });
        makeRustyMove();
    } else {
        board.set({
            orientation: "white"
        });
    }
}
