import express from 'express';
import fs from 'fs';
import path from 'path';
const fileUpload = require('express-fileupload');


function deleteFiles(files) {
    files.forEach(({ tempFilePath }) => {
        fs.unlinkSync(tempFilePath);
    });
}

const app = express();
const port = 4444;
const data_dir = process.env.DATA_DIR || path.join(__dirname, '..', 'tmp');
const adminPasswdPath = path.join(data_dir, '.admin-password')
const foundryvttZipPath = process.env.FOUNDRYVTT_TMP_PATH || path.join(data_dir, 'foundryvtt.zip')

app.use(fileUpload({
    useTempFiles : true,
    tempFileDir : path.join(data_dir, 'tmp_uploads')
}));


app.get('/uploader', (req, res) => res.sendFile(path.join(__dirname, 'index.html')));
app.post('/uploader', (req, res) => {
    const savedSecret = fs.readFileSync(adminPasswdPath, {encoding: 'utf-8'});
    if (req.body && req.body['admin-secret'] && req.body['admin-secret'] === savedSecret) {
        fs.copyFileSync(req.files.foundryvtt.tempFilePath, foundryvttZipPath)
        res.send('Completed');
        return process.exit(0);
    }
    else {
        if (req.files && req.files.length > 0) {
            deleteFiles(req.files);
        }
        return res.status(403).send('Unauthorized');    
    }   
})
app.listen(port, "0.0.0.0", () => {
    const adminPasswd = require('crypto').randomBytes(64).toString('hex');
    fs.writeFileSync(adminPasswdPath, adminPasswd);
    console.log(`Please visit http://localhost:${port}/uploader\nPlease Check here for the admin password: ${adminPasswdPath}`)
});
