import makeWASocket, {
  useMultiFileAuthState,
  fetchLatestBaileysVersion,
  DisconnectReason,
} from "@whiskeysockets/baileys";
import fetch from "node-fetch";
import { Buffer } from "buffer";
import qrcode from "qrcode-terminal";

const startSock = async () => {
  const { state, saveCreds } = await useMultiFileAuthState("auth");
  const sock = makeWASocket({
    version: (await fetchLatestBaileysVersion()).version,
    auth: state,
  });
  sock.ev.on("creds.update", saveCreds);

  sock.ev.on("connection.update", (update) => {
    const { connection, lastDisconnect, qr } = update;
    if (qr) qrcode.generate(qr, { small: true });
    if (connection === "close") {
      const shouldReconnect =
        (lastDisconnect?.error as any)?.output?.statusCode !==
        DisconnectReason.loggedOut;
      console.log(
        `connection closed due to ${lastDisconnect?.error} reconnecting ${shouldReconnect}`
      );
      if (shouldReconnect) {
        startSock();
      }
    } else if (connection === "open") {
      console.log("✅ connection opened");
    }
  });

  sock.ev.on("messages.upsert", async ({ messages }) => {
    const msg = messages[0];
    // console.log("\nMessage payload:", msg);
    try {
      const response = await fetch("http://localhost:8001/api/bot", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(msg),
      });
      const res: any = await response.json();
      // console.log("response api", res);

      if (res.type === "Text") {
        await sock.sendMessage(msg.key.remoteJid!, { text: res.text });
      } else if (res.type === "Sticker") {
        await sock.sendMessage(msg.key.remoteJid!, {
          sticker: Buffer.from(res.buffer, "base64"),
        });
      }
    } catch (err) {
      console.error(`❌ Error send to Axum: ${err}`);
    }
  });
};

startSock();
