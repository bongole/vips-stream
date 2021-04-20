const addon = require('./index.js')
const fs = require('fs');

function test() {
    return new Promise((res) => {
        const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");
        count = 0
        addon.readBufTest(async (_err, tx, read_size) => {
            if (read_stream.readableEnded) {
                console.log('end')
                addon.registerReadBufTest(tx, null)
                res(count)
            }

            buf = await new Promise((res) => read_stream.once('readable', () => {
                const b = read_stream.read(read_size)
                res(b)
            }))

            count += buf.length
            addon.registerReadBufTest(tx, buf)
        })
    })
}

function showMemUsage() {
    const used = process.memoryUsage();
    for (let key in used) {
        console.log(`${key} ${Math.round(used[key] / 1024 / 1024 * 100) / 100} MB`);
    }
}

(async () => {
    showMemUsage()
    for( let i = 0; i < 100; i++){
        let r = await test()
        console.log(r)
        showMemUsage()
        console.log('=============')
    }
    showMemUsage()
})()