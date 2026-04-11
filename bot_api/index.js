import { Client } from "@haelp/teto";
import * as dotenv from "dotenv";
import { createWriteStream } from "fs";
import { format } from "util";
import express from "express";
import cors from "cors";
import * as crypto from 'crypto';
import * as child_process from "child_process"

import { handleChat } from "./src/chat.js";
import { handlePlay } from "./src/play.js";
import { settingsSpectate, roomCheck } from "./src/utils.js";

dotenv.config({ quiet: true });

const app = express();
const port = 3000;

app.use(cors());
app.use(express.json())

let allGames = [];

app.get('/data', (req, res) => {
    res.json(allGames.map(filterGameData));
});
app.post('/submit-data', (req, res) => {
    const { funcStr } = req.body;
    const func = Function("return " + funcStr)();
    try {
        func(allGames);
        res.status(200).json({ message: 'func received successfully' });
    } catch (err) {
        res.status(400).json({ message: err.toString() });
        console.error(err);
    }
})
app.listen(port, _ => console.log(`Server running at http://localhost:${port}`));

const logFile = createWriteStream("./tetr.log", { flags: 'a' });

const logMsg = msg => " --- " + (new Date()).toISOString() + " --- \n" + format(msg);
const logWrite = msg => {
    logFile.write(logMsg(msg) + "\n");
    return msg;
}
const logPrint = msg => {
    logWrite(msg + "\n");
    console.log(logMsg(msg));
    return msg;
}
const debugPrint = msg => {
    logWrite(" --- DEBUG ---\n" + msg + "\n");
    return msg;
}
const logError = msg => {
    let e = " --- ❌ ERROR ❌ --- \n" + msg + "\n"
    logWrite(e);
    console.error(logMsg(e));
    return msg;
}
const logFuncs = { logWrite, logPrint, debugPrint, logError };

(async function () {
    const client = await Client.create({
        username: process.env.TETRIO_USERNAME,
        password: process.env.TETRIO_PASSWORD
    });

    logPrint("✅ Root client connect was successful!");

    client.on("social.invite", async data => {
        if (allGames.length > 4) {
            logError(`Too many people using the bot now, rejected invite from ${data.sender}`);
            return;
        }

        let roomCode = data.roomid.toLowerCase();
        if (roomCode == "x-qp" || roomCode == "x-royale" || roomCode.startsWith("mm-")) {
            logError(`Invite to unauthorized room, rejected invite from ${data.sender}`);
            return;
        }

        const gameData = {
            id: Date.now() + Math.random(), // please please please dont collide
            client: undefined,
            settings: undefined,
            tickData: undefined
        };
        gameData.clear = _ => {
            if (!allGames.some(x => x.id == gameData.id)) {
                logError("Could not find data in allGames!")
            }
            allGames = allGames.filter(x => x.id != gameData.id);
        }
        allGames.push(gameData);

        logPrint(`↗️ Joined room ${roomCode}`);

        await spawnClient(roomCode, gameData);
    });
})();

async function spawnClient(roomCode, gameData) {
    const client = await Client.create({
        username: process.env.TETRIO_USERNAME,
        password: process.env.TETRIO_PASSWORD,
        ribbon: { transport: "json" }
    });

    const bot_engine = {
        engine: child_process.spawn("../target/release/keygen"),
        keyInfo: {}
    };
    bot_engine.engine.stdout.on("data", data => {
        data = JSON.parse(data.toString().trim());
        bot_engine.keyInfo.allKeys = data.keys;
        bot_engine.keyInfo.length = data.keys.length;
        bot_engine.keyInfo.sendingStdin = false,
        bot_engine.keyInfo.desiredLocation = {
            piece: data.desired_location.piece,
            x: data.desired_location.x,
            y: data.desired_location.y,
            rotation: {"Up": 0, "Right": 1, "Down": 2, "Left": 3}[data.desired_location.rotation]
        };
    });

    bot_engine.engine.stderr.on("data", data => {
        logError(data.toString());
        keyInfo.error = data.toString();
    });

    client._destroy = client.destroy;
    client.destroy = async _ => {
        await client?.room?.leave();
        await client._destroy();
        logPrint(`🚪 Left room ${room?.id || roomCode}`);
        bot_engine.engine.kill();
        bot_engine.engine.stdout.removeAllListeners("data");
        gameData.clear();
    }

    let room;
    try {
        room = await client.rooms.join(roomCode);
    } catch (e) {
        logError(`Could not join room ${roomCode}`);
        client.destroy();
        return;
    }
    
    if (room.type == "system") {
        await client.destroy();
        return;
    }

    const settings = {
        enabled: false,
        pps: 1,
        turnbased: 0,
        attitude: "default",
        finesse: "inhuman"
    };

    gameData.client = client;
    gameData.settings = settings;

    room._chat = room.chat;
    room.chat = m => {
        room._chat(m);
        debugPrint(`💬 Sent message in ${room?.id}. Content: ${JSON.stringify(m)}`);
    }
    room.msg = msgObj => room.chat(msgObj[settings.attitude]);

    await settingsSpectate(room, settings);

    client.on("room.chat", dt => handleChat(dt, client, room, settings, logFuncs));
    client.on("client.room.kick", async _ => await client.destroy());
    client.on("client.game.start", _ => {
        if (Object.keys(roomCheck(room)).length) {
            room.chat("INVALID SETTINGS");
            client.destroy();
            return;
        }
        room.chat("never say glhf");
    });
    client.on("client.game.end", _ => {
        room.chat("never say gg");
        gameData.tickData = undefined;
    });
    client.on("client.game.round.start", data => {
        handlePlay(data, client, room, settings, gameData, logFuncs, bot_engine);
    });
    client.on("room.update", async _ => {
        if (settings.enabled) await settingsSpectate(room, settings);
        else room.switch("spectator");
    });
    client.on("room.update.host", _ => {
        if (room.owner == client.user.id) client.destroy();
    });
    client.on("room.update.bracket", async dt => {
        if (dt.uid != client.user.id) return;
        if ((dt.bracket == "player") != settings.enabled) {
            if (settings.enabled) await settingsSpectate(room, settings);
            else room.switch("spectator");
        }
    })
}

process.on("uncaughtException", err => {
    logError(err);
    const id = crypto.randomBytes(8).toString("hex");
    try {
        logWrite(`ERROR (code ${id})${allGames.length ? ", left rooms" + allGames.map(x => x?.client?.room?.roomid).join(", ") : ""}\n${err.toString()}`);
        for (const game of allGames) {
            game.client.room.chat(`SHOOT something REALLY BAD went wrong so i gtg, if u need to report this, tell chadhary_12345 (tyrcnex on discord) or kimjoohyeon_ that the error code is ${id}`);
            game.client.destroy();
        }
    } catch (err) {
        logError(err);
    }
    allGames = [];
});

process.on("SIGINT", async _ => {
    (async function() {
        for await (const game of allGames) {
            game.client.room.chat(`dev turned off bot, goodbye!`);
            await game.client.destroy();
        }
    })().finally(_ => process.exit(0));

    setTimeout(process.exit, 5000)
})

function filterGameData(data) {
    return {
        id: data.id,
        client: !data.client ? undefined : {
            ...keepKeys(data.client, ["user", "disconnected", "handling", "dead"]),
            room: keepKeys(data.client.room, ["id", "type", "name", "name_safe", "owner", "creator", "autostart", "match", "options", "chats"])
        },
        settings: data.settings,
        tickData: data.tickData
    }
}

function keepKeys(obj, keys) {
    if (!obj) return {};
    const newObj = {};
    for (const key of keys) {
        newObj[key] = obj[key];
    }
    return newObj;
}