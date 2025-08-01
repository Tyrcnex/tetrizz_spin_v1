const requiredOptions = {
    g: 0,
    gincrease: 0,
    spinbonuses: "all+",
    kickset: "SRS+",
    room_handling: false,
    allow_harddrop: true,
    boardwidth: 10,
    boardheight: 20
}

const emojiMinos = {
    "i": "🟫",
    "o": "🟨",
    "j": "🟦",
    "l": "🟧",
    "s": "🟩",
    "z": "🟥",
    "t": "🟪",
}

export function drawEngine(engine) {
    let all_str = [];
    let board_new = [0,0,0,0,0,0,0,0,0,0]
    for (let y = 19; y >= 0; y--) {
        let str = "";
        for (let x = 0; x < 10; x++) {
            let mino = engine.board.state[y][x]
            str += mino ? (emojiMinos[mino.toString().toLowerCase()] || "⬛️") : "⬜️";
            board_new[x] += +!!mino * (1 << y);
        }
        all_str.push(str);
    }
    all_str[5] += `          b2b:         ${engine.stats.b2b + 1}`;
    all_str[6] += `          queue:       ${engine.held + "," + engine.queue.value.join(",")}`;
    all_str[7] += `          board state: [${board_new.toString()}]`;
    return all_str.join("\n");
}

export function tryUnspectate(room, settings) {
    try {
        room.switch("player");
        settings.enabled = true;
    } catch (err) {
        room.chat("error occured when trying to unspectate, room is probably full");
        settings.enabled = false;
    }
}

export function roomCheck(room) {
    let allInvalid = {};
    for (const [key, value] of Object.entries(requiredOptions)) {
        if (room.options[key] != value) {
            allInvalid[key] = value;
        }
    }
    return allInvalid;
}

export function settingsSpectate(room, settings) {
    let invalidOptions = roomCheck(room);
    if (!Object.keys(invalidOptions).length) {
        tryUnspectate(room, settings);
    } else {
        let cmd = `/set ${Object.entries(invalidOptions).map(([k, v]) => `options.${k}=${v}`).join("; ")}`;
        room.msg({
            default: `Invalid settings, please run the following command:\n\n${cmd}`,
            cute: `im scawed of these settings, m-master could you pwease run this:\n\n${cmd}`,
            bot: `Invalid settings. The following command may aid in fixing the issue:\n\n${cmd}`,
            chadhary: `wrong settings bro run this:\n\n${cmd}`
        });
        room.switch("spectator");
        settings.enabled = false;
    }
}