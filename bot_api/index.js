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

const logMsg = msg => " --- " + (new Date()).toUTCString() + " ---\n" + format(msg);
const logWrite = msg => {
    logFile.write(logMsg(msg) + "\n");
    return msg;
}
const logPrint = msg => {
    logWrite(msg + "\n");
    console.log(logMsg(msg));
    return msg;
}
const logFuncs = { logWrite, logPrint };

(async function () {
    const client = await Client.connect({
        username: process.env.TETRIO_USERNAME,
        password: process.env.PASSWORD
    });

    console.log("Root client connect was successful!");

    client.on("social.dm", async dm => {
        client.social.dm(dm.data.user, "hi, i cant read ur message, but if you want to use commands run %help (it's a percent sign). you cant run it in dms (this is todo), so run it after u invite me to a room!")
            .catch(err => undefined);
    });

    client.on("social.invite", async data => {
        // // maintenance
        // if (data.sender == "64eee0bd4edc0179c6e85b12") {
        //     // client.social.dm(data.sender, "hi owner");
        // } else {
        //     client.social.dm(data.sender, "sorry! im under maintenance rn, pls try again later :3");
        //     return;
        // }

        if (allGames.length > 4) {
            client.social.dm(data.sender, "sorry! too many ppl are using the bot rn, ull have to wait for a bit, mb gang ill be back soon")
                .catch(err => undefined);
            return;
        }

        let roomCode = data.roomid.toLowerCase();
        if (roomCode == "x-qp" || roomCode == "x-royale" || roomCode.startsWith("mm-")) {
            client.social.dm(data.sender, "dont invite me to this room >:(");
            return;
        }

        const gameData = {
            id: Date.now() + Math.random(), // please please please dont collide
            client: undefined,
            settings: undefined,
            tickData: undefined
        };
        gameData.clear = _ => {
            console.assert(allGames.some(x => x.id == gameData.id), "Could not find data in allGames!");
            allGames = allGames.filter(x => x.id != gameData.id);
        }
        allGames.push(gameData);

        logPrint(`Joined room ${roomCode}`);

        await spawnClient(roomCode, gameData);
    });
})();

async function spawnClient(roomCode, gameData) {
    const client = await Client.connect({
        username: process.env.TETRIO_USERNAME,
        password: process.env.PASSWORD
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
        console.error(data.toString());
        keyInfo.error = data.toString();
    });

    client._destroy = client.destroy;
    client.destroy = async _ => {
        logPrint(`Left room ${room?.id || roomCode}`);
        await client._destroy();
        bot_engine.engine.kill();
        bot_engine.engine.stdout.removeAllListeners("data");
        gameData.clear();
    }

    let room;
    try {
        room = await client.rooms.join(roomCode);
    } catch (e) {
        console.error("could not join room!!");
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

    room.msg = msgObj => room.chat(msgObj[settings.attitude]);

    await settingsSpectate(room, settings);

    client.on("room.chat", dt => handleChat(dt, client, room, settings));
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
    console.error(err);
    const id = crypto.randomBytes(8).toString("hex");
    try {
        logWrite(`ERROR (code ${id})${allGames.length ? ", left rooms" + allGames.map(x => x?.client?.room?.roomid).join(", ") : ""}\n${err.toString()}`);
        for (const game of allGames) {
            game.client.room.chat(`SHOOT something REALLY BAD went wrong so i gtg, if u need to report this, tell chadhary_12345 (tyrcnex on discord) that the error code is ${id}`);
            game.client.destroy();
        }
    } catch (err) {
        console.error(err);
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