import fs from "fs";
export function deleteFiles(files) {
  files.forEach(({ tempFilePath }) => {
    fs.unlinkSync(tempFilePath);
  });
}
