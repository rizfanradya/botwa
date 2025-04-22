"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const baileys_1 = __importStar(require("@whiskeysockets/baileys"));
const node_fetch_1 = __importDefault(require("node-fetch"));
const buffer_1 = require("buffer");
const qrcode_terminal_1 = __importDefault(require("qrcode-terminal"));
const startSock = () => __awaiter(void 0, void 0, void 0, function* () {
    const { state, saveCreds } = yield (0, baileys_1.useMultiFileAuthState)("auth");
    const sock = (0, baileys_1.default)({
        version: (yield (0, baileys_1.fetchLatestBaileysVersion)()).version,
        auth: state,
    });
    sock.ev.on("creds.update", saveCreds);
    sock.ev.on("connection.update", (update) => {
        var _a, _b;
        const { connection, lastDisconnect, qr } = update;
        if (qr)
            qrcode_terminal_1.default.generate(qr, { small: true });
        if (connection === "close") {
            const shouldReconnect = ((_b = (_a = lastDisconnect === null || lastDisconnect === void 0 ? void 0 : lastDisconnect.error) === null || _a === void 0 ? void 0 : _a.output) === null || _b === void 0 ? void 0 : _b.statusCode) !==
                baileys_1.DisconnectReason.loggedOut;
            console.log(`connection closed due to ${lastDisconnect === null || lastDisconnect === void 0 ? void 0 : lastDisconnect.error} reconnecting ${shouldReconnect}`);
            if (shouldReconnect) {
                startSock();
            }
        }
        else if (connection === "open") {
            console.log("✅ connection opened");
        }
    });
    sock.ev.on("messages.upsert", (_a) => __awaiter(void 0, [_a], void 0, function* ({ messages }) {
        const msg = messages[0];
        try {
            const response = yield (0, node_fetch_1.default)("http://localhost:8001/api/bot", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(msg),
            });
            const res = yield response.json();
            if (res.type === "Text") {
                yield sock.sendMessage(msg.key.remoteJid, { text: res.text });
            }
            else if (res.type === "Sticker") {
                yield sock.sendMessage(msg.key.remoteJid, {
                    sticker: buffer_1.Buffer.from(res.buffer, "base64"),
                });
            }
        }
        catch (err) {
            console.error(`❌ Error send to Axum: ${err}`);
        }
    }));
});
startSock();
