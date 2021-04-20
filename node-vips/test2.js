const fs = require('fs');

const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");
let count = 0
async function test(read_size) {
    if(read_stream.readableEnded) return null

    buf = await new Promise((res) => read_stream.once('readable', () => {
        const b = read_stream.read(read_size)
        res(b)
    }))

    if( buf ) count += buf.length

    return buf ?  buf.length : null
}

(async () => {
    console.log('start')
    await new Promise((r) => setTimeout(r, 10000))
    while( await test(4096) !== null ){}
    console.log(count)
    console.log('stop')
    global.gc()
    await new Promise((r) => setTimeout(r, 30000))
})()
