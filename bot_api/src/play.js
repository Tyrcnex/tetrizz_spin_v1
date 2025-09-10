import * as dotenv from "dotenv";

dotenv.config({ path: "../.env", quiet: true });

// moveLeft, moveRight, softDrop, hardDrop, hold, rotateCW, rotateCCW, rotate180

export async function handlePlay(data, client, room, settings, gameData, logFuncs, bot_engine) {
    bot_engine.keyInfo = {
        sendingStdin: false,
        allKeys: [],
        desiredLocation: {},
        startFrame: 0,
        length: 0,
        error: ""
    };

    const { logWrite, logPrint } = logFuncs;
    const [tick, engine, allPlayers] = data; // allPlayers is all engines, for example [{name:'user1',gameid:1,engine:[Engine]}, {name:'user2',gameid:2,engine:[Engine]}]

    const opponentEngine = allPlayers.filter(x => x.name != process.env.USERNAME)[0].engine;
    const additionalBoardInfo = {
        lastb2b: 0,
        b2bDeficit: 0
    }

    tick(async dt => {
        gameData.tickData = dt;
        if (bot_engine.keyInfo.error.length > 0) {
            room.chat("nooo! there was a problem with the keyfinder :("); // maybe do log handling here
            logWrite(bot_engine.keyInfo.error);
            return {
                keys: Array(20).fill([
                    keydown("hardDrop", dt.frame),
                    keyup("hardDrop", dt.frame),
                ]).flat()
            }
        }

        if (dt.frame == 0) {
            return {
                keys: [
                    keydown("hold", 0),
                    keyup("hold", 0),
                    keydown("hardDrop", 0),
                    keyup("hardDrop", 0)
                ]
            }
        }
        if (dt.frame <= 5) return {};

        if (bot_engine.keyInfo.allKeys.length == 0 && !bot_engine.keyInfo.sendingStdin && (!settings.turnbased || engine.stats.pieces < settings.turnbased * Math.floor(opponentEngine.stats.pieces / settings.turnbased))) {
            bot_engine.keyInfo.startFrame = dt.frame; // to fudge a bit
            bot_engine.keyInfo.sendingStdin = true;

            let board_new = [0,0,0,0,0,0,0,0,0,0];
            for (let row = 0; row < 20; row++) {
                for (let col = 0; col < 10; col++) {
                    board_new[col] += +!!engine.board.state[row][col] * (1 << row);
                }
            }

            let queue = [engine.falling.symbol].concat(engine.queue.value).map(x => x.toUpperCase());
            let hold = engine.held.toUpperCase(); // if this errors, u fucking suck because the bot should have held already

            if (additionalBoardInfo.lastb2b == engine.stats.b2b) additionalBoardInfo.b2bDeficit += 1;
            else additionalBoardInfo.b2bDeficit = 0;

            let input = {
                game: {
                    board: { cols: board_new },
                    hold,
                    b2b: engine.stats.b2b + 1,
                    b2b_deficit: 0 // remove additionalBoardInfo.b2bDeficit
                },
                queue,
                beam_width: Math.floor(3500 / settings.pps),
                beam_depth: 14
            };

            bot_engine.engine.stdin.write(JSON.stringify(input) + "\n");
        }

        if (settings.finesse == "inhuman") {
            if (bot_engine.keyInfo.allKeys.length != 0 && dt.frame - bot_engine.keyInfo.startFrame > 60 / settings.pps - bot_engine.keyInfo.length - 1) {
                let key = bot_engine.keyInfo.allKeys.shift();
                // if (bot_engine.keyInfo.allKeys.length == 0) {
                //     bot_engine.keyInfo.length == 0;
                //     if (key != "hardDrop") logPrint("Keyfinder error: hardDrop assert failed");
                //     const shiftedLoc = centerToCorner(bot_engine.keyInfo.desiredLocation);
                //     const tetrioLoc = {
                //         piece: engine.falling.symbol.toUpperCase(),
                //         x: engine.falling.x,
                //         y: engine.falling.y,
                //         rotation: engine.falling.rotation
                //     };
                //     if (!(
                //         shiftedLoc.piece == tetrioLoc.piece
                //         && shiftedLoc.x == tetrioLoc.x 
                //         && shiftedLoc.y == tetrioLoc.y
                //         && shiftedLoc.rotation == tetrioLoc.rotation
                //     )) logPrint(`Keyfinder error: expected ${JSON.stringify(shiftedLoc)}, got ${JSON.stringify(tetrioLoc)}`);
                // }
                return { keys: [
                    keydown(key, dt.frame),
                    keyup(key, dt.frame, 0.2 * (key == "softDrop"))
                ] };
            }
        } else {
            if (bot_engine.keyInfo.allKeys.length != 0 && dt.frame - bot_engine.keyInfo.startFrame > 60 / settings.pps - 1) {
                let keys = [];
                let sfr = 0;
                for (const key of bot_engine.keyInfo.allKeys) {
                    keys.push(keydown(key, dt.frame, sfr));
                    if (key == "softDrop") sfr += 0.1;
                    keys.push(keyup(key, dt.frame, sfr));
                }
                bot_engine.keyInfo.allKeys = [];
                bot_engine.keyInfo.length = 0;
                return { keys };
            }
        }

        return {};
    });
}

// my movegen defines a location as the center of the piece
// tetrio defines it as the top left corner
// this function shifts from my movegen to tetrio's version
function centerToCorner(pieceLocation) {
    let xOff, yOff;
    if (pieceLocation.piece == "I") {
        xOff = [-1,-2,-2,-1][pieceLocation.rotation];
        yOff = [1,1,2,2][pieceLocation.rotation];
    } else if (pieceLocation.piece == "O") {
        xOff = [0,0,-1,-1][pieceLocation.rotation];
        yOff = [1,0,0,1][pieceLocation.rotation];
    } else {
        xOff = -1;
        yOff = 1;
    }
    return {
        ...pieceLocation,
        x: pieceLocation.x + xOff,
        y: pieceLocation.y + yOff,
    }
}

function keydown(key, frame, subframe = 0) {
    return {
        frame: frame,
        type: "keydown",
        data: {
            key,
            subframe
        }
    }
}

function keyup(key, frame, subframe = 0) {
    return {
        frame: frame,
        type: "keyup",
        data: {
            key,
            subframe
        }
    }
}