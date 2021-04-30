const addon = require('./index.js');
const fs = require('fs');

function test(){
    const stream = fs.createReadStream('/home/bongole/image/4k.jpg', { highWaterMark: 128 * 1024 })
    //const buflist = new addon.BufferList(4 * 1024 * 1024);
    const buflist = new addon.BufferList();
    stream.on('data', (buf) => {
        let r = buflist.push(buf)
    })
}

for( let i = 0; i < 100; i++ ){
    test()
}
