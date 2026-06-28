import QRCode from "qrcode";

export async function createQrCode(shortUrl: string) {
  return QRCode.toDataURL(shortUrl, {
    color: {
      dark: "#0E0E0E",
      light: "#FFFFFF",
    },
    margin: 2,
    width: 192,
  });
}