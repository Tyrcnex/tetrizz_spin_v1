import * as dotenv from "dotenv";
import { settingsSpectate } from "./utils.js";
import { writeFileSync } from "fs";

dotenv.config({ path: "../.env", quiet: true });

function hostOnly(room, msgData) {
    if (msgData.user._id != room.owner && msgData.user._id != process.env.OWNER_ID) {
        room.msg({
            default: "This command is restricted to host level.",
            cute: "w-what? b-but ur not the owner- i mean, host...",
            bot: "Permission denied: command is only available to host.",
            chadhary: "ur not host bro u cant use this cmd"
        });
        return false;
    }
    return true;
}

const defaultCommands = {
    pps: {
        aliases: ["p"],
        hostonly: true,
        description: "changes the pps of the bot",
        usage: "usage: %pps [pps value]\n\nexamples:\n - pps 2\n - pps 4.2",
        exec: (client, room, settings, msgData, args) => {
            let pps = args[0];
            if (isNaN(pps)) {
                room.chat("invalid pps");
                return;
            }
            pps = +pps;
            if (pps < 0)
                room.msg({
                    default: "unable to set PPS below 0.",
                    cute: "sorry... i can't steal pieces from the board like you stole my mechanical heart </3",
                    bot: "Input outside specified operating parameters. Unable to compute inverse finesse.",
                    chadhary: "do you want me to take away pieces from the board what",
                });
            else if (pps < 0.5)
                room.msg({
                    default: "PPS is too low. (Bot operates at minimum 0.5 PPS)",
                    cute: "we... c-can take it slow later... entertain me a bit more :c",
                    bot: "Input outside specified operating parameters. Unable to calculate any slower pace.",
                    chadhary: "too slow bro im not f rank",
                });
            else if (pps > 5)
                room.msg({
                    default: "PPS is too high. (Bot operates at maximum 5 PPS)",
                    cute: "a-aren't we taking this too fast!? be nice t-to me... i'm not ready yet ><",
                    bot: "Input outside specified operating parameters. Insufficient computation power.",
                    chadhary: "too fast bro im not mochbot",
                });
            else {
                settings.pps = pps;
                const pps3note = pps >= 3 ? "\n\nnote from dev: in-game pps might be unreliable for pps above 3" : "";
                room.msg({
                    default: `Set PPS to ${pps}.${pps3note}`,
                    cute: `yay! my pps is ${pps}, thx! :3${pps3note}`,
                    bot: `Operation successful: PPS set to ${pps}${pps3note}`,
                    chadhary: `ok i set pps to ${pps}${pps3note}`
                });
            }
        },
    },
    kill: {
        aliases: ["leave"],
        hostonly: true,
        description: "disconnects the bot from the room",
        usage: "usage: %kill",
        exec: (client, room, settings, msgData, args) => {
            room.msg({
                default: "Disconnecting from room. Goodbye!",
                cute: "call me when you're in the mood to *play* again :3",
                bot: "All processes terminated. Powering down.",
                chadhary: "*gasp* *choke* why would you... *dies*",
            });
            client.destroy();
        },
    },
    unspectate: {
        aliases: ["enable"],
        hostonly: true,
        description: "tries to unspectate and set the bot to a player",
        usage: "usage: %unspectate",
        exec: (client, room, settings, msgData, args) => {
            settingsSpectate(room, settings);
        },
    },
    spectate: {
        aliases: ["disable"],
        description: "makes the bot a spectator",
        usage: "usage: %spectate",
        hostonly: true,
        exec: (client, room, settings, msgData, args) => {
            room.switch("spectator");
            settings.enabled = false;
            room.msg({
                default: `Now in spectate mode.`,
                cute: `im not gonna play now... but i can still watch! :>`,
                bot: `Spectate mode activated.`,
                chadhary: `aight i spectated now go have fun without me`
            });
        },
    },
    settings: {
        aliases: ["config", "s"],
        hostonly: false,
        description: "view or change settings of the bot",
        usage: "usage:\n - %settings\n - %settings [setting]\n - %settings [setting] [value]\n\nexamples:\n - %settings\n - %settings turnbased\n - %settings turnbased 2",
        exec: (client, room, settings, msgData, args) => {
            if (!args.length) {
                room.chat(`Settings:\n - PPS: ${settings.pps}\n - Turn-based (turnbased): ${settings.turnbased == 0 ? "off" : settings.turnbased}\n - Attitude: ${settings.attitude}\n - Finesse: ${settings.finesse}`);
                return;
            }
            if (args.length == 1) {
                let setting = args[0]?.toLowerCase();
                if (setting == "turnbased" || setting == "turn_based" || setting == "turn-based") {
                    room.msg({
                        default: "If this mode is on, after you play some moves, I'll play the same number of moves.",
                        cute: "OO TURNBASED I LOV TURNBASED so turnbased is like first i place some pieces and then u place some pieces!! so fun!!! :>>>",
                        bot: "Turn-based documentation: Players take turns placing n pieces at a time.",
                        chadhary: "turn-based mode: i place n pieces, u place n pieces. chesstris basically"
                    });
                } else if (setting == "pps") {
                    room.msg({
                        default: "How fast the bot goes (pieces per second).",
                        cute: "this tells me how fast i should go! but dont make me go too fast, be n-nice...",
                        bot: "Pieces per second is a statistic that measures how fast I place pieces.",
                        chadhary: "pps self explanatory bro"
                    });
                } else if (setting == "attitude") {
                    room.chat("sets my attitude ;)");
                }
                return;
            }

            if (!hostOnly(room, msgData)) return;
            
            let setting = args[0]?.toLowerCase();
            let value = args[1]?.toLowerCase();
            if (setting == "turnbased" || setting == "turn_based" || setting == "turn-based") {
                if (isNaN(value)) {
                    room.chat("invalid value (must be number)");
                    return;
                }
                value = +value;
                if (value % 1 != 0) {
                    room.chat("value must be integer");
                    return;
                }
                if (value < 0 || value > 10) {
                    room.chat("value must be between 0 and 10");
                    return;
                }
                settings.turnbased = value;
                if (value == 0) room.chat("turned turnbased mode off");
                else room.chat(`set turnbased moves to ${value}`);
            } else if (setting == "pps") {
                room.chat("please use pps command instead");
            } else if (setting == "attitude") {
                room.chat("please use attitude command instead");
            } else if (setting == "finesse") {
                const validFinesse = ["inhuman", "bot"];
                if (validFinesse.includes(value)) {
                    settings.finesse = value;
                    room.chat(`set finesse to ${value}!`);
                } else {
                    room.chat(`invalid finesse ${value}`);
                }
            }
        },
    },
    attitude: {
        aliases: ["personality"],
        hostonly: false,
        description: "change the attitude/personality of the bot",
        usage: "usage: %attitude [attitude type]\n\nexamples:\n - %attitude cute\n - %attitude depressed",
        exec: (client, room, settings, msgData, args) => {
            const validAttitudes = ["bot", "default", "cute", "chadhary"];
            let attitude = args[0]?.toLowerCase();
            if (validAttitudes.includes(attitude)) {
                settings.attitude = attitude;
                room.chat(`attitude set to ${attitude}!\nthis is still a work in progress, some messages don't have special attitudes :(`);
            } else {
                room.chat(`invalid attitude, valid attitudes are the following:\n${validAttitudes.map((x) => ` - ${x}`).join("\n")}`);
            }
        },
    },
    savereplay: {
        aliases: ["replay"],
        hostonly: false,
        description: "dev only command to save replays",
        usage: "usage: %savereplay",
        exec: (client, room, settings, msgData, args) => {
            if (msgData.user._id != process.env.OWNER_ID) {
                room.chat("this is a dev only command");
                return;
            }
            if (args.length > 1) {
                room.chat("the extra part of the filename cannot have spaces! use underscores or dashes")
            }
            const replay = client.room.replay?.export();
            if (!replay) {
                room.chat("no replay found!");
                return;
            }
            const filename = `${(new Date()).toJSON().slice(0,19).replace("T", "_").replaceAll(":", "-")}${args.length ? "__" + args[0] : ""}.ttrm`;
            writeFileSync(`../replays/${filename}`, JSON.stringify(replay));
            room.chat(`saved replay to ${filename}!`);
        }
    }
};

export async function handleChat(data, client, room, settings) {
    if (data.user.username == process.env.TETRIO_USERNAME) return;

    let content = data.content.trim();
    if (content.toLowerCase().replace(/[^0-9a-z]/g, "").match(/(g|good)(l|luck|1)(h|have)(f|fun)/g)) {
        room.chat("never say glhf");
    }

    if (content.toLowerCase().replace(/[^0-9a-z]/g, "").match(/(g|good)(g|game)/g)) {
        room.chat("never say gg");
    }

    let prefix = content.toLowerCase().match(new RegExp(`^(%|${process.env.TETRIO_USERNAME} |@${process.env.TETRIO_USERNAME} )`));
    if (!prefix) return;
    let args = content.slice(prefix[0].length).trim().split(" ");
    let userCmd = args?.shift()?.toLowerCase();

    const commands = { ...defaultCommands, ...{} }; // todo make attitude specific commands
    commands["help"] = {
        aliases: [],
        hostonly: false,
        exec: (client, room, settings, msgData, args) => {
            if (!args.length) {
                room.chat(
                    `Available commands: ${Object.keys(commands)
                        .map((x) => "%" + x)
                        .join(", ")}`,
                );
                return;
            }
            let cmd = args[0]?.toLowerCase();
            let h;
            if (cmd == "help") {
                room.chat(`--- command %help ---\nhelps you with commands\nusage:\n - %help\n - %help [command name]\n\nexamples:\n - %help\n - %help pps`);
            } else if (Object.keys(commands).find(x => x == cmd)) {
                let hcmdObj = commands[cmd];
                room.chat(`--- command %${cmd} ---\n${hcmdObj.description}\n${hcmdObj.usage}`);
            } else if (h = Object.values(commands).find(x => x.aliases.includes(cmd))) {
                room.chat(`--- command %${cmd} ---\n${h.description}\n${h.usage}`);
            } else {
                room.chat(`unknown command ${cmd}`);
                return;
            }
        },
    };

    let cmdObj;

    if (Object.keys(commands).find(x => x == userCmd)) {
        cmdObj = commands[userCmd];
    } else if (cmdObj = Object.values(commands).find(x => x.aliases.includes(userCmd))) {
    } else {
        room.chat(`unknown command ${userCmd}`);
        return;
    }

    if (cmdObj.hostonly && !hostOnly(room, data)) {
        return;
    }
    cmdObj.exec(client, room, settings, data, args);
}