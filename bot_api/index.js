import { Client } from "@haelp/teto";
import * as dotenv from "dotenv";
import { createWriteStream } from "fs";
import { format } from "util";
import express from "express";
import cors from "cors";
import * as crypto from 'crypto';
import cloneDeep from "lodash.clonedeep";

import { handleChat } from "./src/chat.js";
import { handlePlay } from "./src/play.js";
import { settingsSpectate, roomCheck } from "./src/utils.js";

dotenv.config({ quiet: true });

const app = express();
const port = 3000;

app.use(cors());

let allGames = [];

app.get('/data', (req, res) => {
    res.json(allGames.map(filterGameData));
});
app.listen(port, _ => console.log(`Server running at http://localhost:${port}`));

const logFile = createWriteStream("./tetr.log", { flags: 'w' });

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
        username: process.env.USERNAME,
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
            tickData: undefined
        };
        gameData.clear = _ => {
            console.assert(allGames.some(x => x.id == gameData.id), "Could not find data in allGames!");
            allGames = allGames.filter(x => x.id != gameData.id);
        }
        allGames.push(gameData);

        logPrint(`Joined room ${roomCode}`);

        spawnClient(roomCode, gameData);
    });
})();

async function spawnClient(roomCode, gameData) {
    const client = await Client.connect({
        username: process.env.USERNAME,
        password: process.env.PASSWORD
    });

    const room = await client.rooms.join(roomCode);

    client._destroy = client.destroy;
    client.destroy = _ => {
        logPrint(`Left room ${room.id}`);
        client._destroy();
        gameData.clear();
    }
    
    if (room.type == "system") {
        client.destroy();
        return;
    }

    gameData.client = client;

    const settings = {
        enabled: false,
        pps: 1,
        turnbased: 0,
        attitude: "default",
    };

    room.msg = msgObj => room.chat(msgObj[settings.attitude]);

    settingsSpectate(room, settings);

    client.on("room.chat", dt => handleChat(dt, client, room, settings));
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
        handlePlay(data, client, room, settings, gameData, logFuncs);
    });
    client.on("room.update", _ => settingsSpectate(room, settings));
    client.on("room.update.host", _ => {
        if (room.owner == client.user.id) client.destroy();
    });
    client.on("room.update.bracket", dt => {
        if (dt.uid != client.user.id) return;
        if ((dt.bracket == "player") != settings.enabled) {
            if (settings.enabled) settingsSpectate(room, settings);
            else room.switch("spectator");
        }
    })
}

process.on("uncaughtException", err => {
    console.error(err);
    // const id = crypto.randomBytes(8).toString("hex");
    // logWrite(`ERROR (code ${id})${allGames.length ? ", left rooms" + allGames.map(x => x?.room?.roomid).join(", ") : ""}\n${err.toString()}`);
    // for (const game of allGames) {
    //     game.room.chat(`SHOOT something REALLY BAD went wrong so i gtg, if u need to report this, tell chadhary_12345 (tyrcnex on discord) that the error code is ${id}`);
    //     game.client.destroy();
    // }
    // allGames = [];
});

function filterGameData(data) {
    return {
        id: data.id,
        client: !data.client ? undefined : {
            ...keepKeys(data.client, ["user", "disconnected", "handling"]),
            room: keepKeys(data.client.room, ["id", "type", "name", "name_safe", "owner", "creator", "autostart", "match", "options"])
        },
        tickData: data.tickData
    }
}

function keepKeys(obj, keys) {
    const newObj = {};
    for (const key of keys) {
        newObj[key] = obj[key];
    }
    return newObj;
}