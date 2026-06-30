import QRCode from "qrcode";

export async function createQrCode(shortUrl: string) {
  return QRCode.toDataURL(shortUrl, {
    color: {
      dark: "#0a0b10",
      light: "#FFFFFF",
    },
    margin: 2,
    width: 192,
  });
}