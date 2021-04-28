const fs = require('fs');
const addon = require('./index.js');

function test(){
    const read_stream = fs.createReadStream('/home/bongole/image/4k.jpg', { highWaterMark: 40 * 1024 })
    addon.readBufTest(40 * 1024, async (_err, ctx, read_size) => {
        if (read_stream.readableEnded) {
            addon.registerReadBufTest(ctx, null)
            return
        }

        let buf = read_stream.read(read_size)
        if( buf === null ){
            buf = await new Promise((res) => read_stream.once('readable', () => {
                const b = read_stream.read(read_size)
                res(b)
            }))
        }

        /*
        let buf = await new Promise((res) => read_stream.once('readable', () => {
            const b = read_stream.read(read_size)
            res(b)
        }))
        */

        addon.registerReadBufTest(ctx, buf)
    })

    addon.readBuf((_err, ctx) => {
        read_stream.on('data', (buf) => {
            let r = addon.registerReadBuf(ctx, buf)
            if( !r ){
                read_stream.pause()
            }
        })
    }, (_err) => {
        read_stream.resume()
    })
}

for( let i = 0; i < 100; i++ ){
    test()
}
