import makeWASocket, {
  useMultiFileAuthState,
  fetchLatestBaileysVersion,
  DisconnectReason,
  downloadMediaMessage,
} from "@whiskeysockets/baileys";
import axios from "axios";
import { Buffer } from "buffer";
import { fileTypeFromBuffer } from "file-type";
import FormData from "form-data";
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
    const urlApi = "http://localhost:8001/api";
    const msg = messages[0];
    if (!msg.message) return;

    const caption =
      msg.message?.imageMessage?.caption ||
      msg.message?.extendedTextMessage?.text ||
      msg.message?.conversation;
    const remoteJid = msg.key.remoteJid!;
    if (!caption) return;

    const command = caption.split(" ")[0].toLowerCase();
    const args = caption.slice(command.length).trim();

    try {
      if (command === "/stiker") {
        const media = msg.message?.imageMessage || msg.message?.videoMessage;
        if (media && args) {
          const file: any = await downloadMediaMessage(msg, "buffer", {});
          const fileType = await fileTypeFromBuffer(file);
          const form: any = new FormData();

          form.append("file", file, {
            filename: `image.${fileType?.ext || "jpg"}`,
            contentType: fileType?.mime || "image/jpeg",
          });
          form.append("text", args);

          const { data } = await axios.post(
            `${urlApi}/img_to_stk_with_txt`,
            form,
            {
              headers: form.getHeaders(),
            }
          );

          await sock.sendMessage(remoteJid, {
            sticker: Buffer.from(data.buffer, "base64"),
          });
        } else if (args) {
          const { data } = await axios.post(`${urlApi}/txt_to_stk`, {
            text: args,
          });
          await sock.sendMessage(remoteJid, {
            sticker: Buffer.from(data.buffer, "base64"),
          });
        } else if (media) {
          const { data } = await axios.post(
            `${urlApi}/img_to_stk`,
            await downloadMediaMessage(msg, "buffer", {}),
            {
              headers: { "Content-Type": "application/octet-stream" },
            }
          );
          await sock.sendMessage(remoteJid, {
            sticker: Buffer.from(data.buffer, "base64"),
          });
        }
      } else if (command === "/menu") {
        const { data } = await axios.get(`${urlApi}/menu`);
        await sock.sendMessage(remoteJid, { text: data.message });
      }
    } catch (err) {
      console.log("❌ Error saat handle command:", err);
    }
  });
};

startSock();
