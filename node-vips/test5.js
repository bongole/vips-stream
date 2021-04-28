const fs = require('fs');

const stream = fs.createReadStream('/home/bongole/image/4k.jpg')
stream.on('data', (buf) => {
    console.log(buf.length)
})
