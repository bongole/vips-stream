const addon = require('./index.js')
const fs = require('fs');

async function test() {
    const stream = fs.createReadStream("/home/bongole/image/4k.jpg");

    let init = false;
    let vips = await addon.createVipsImage((err, ctx, read_size) => {
        if (stream.readableEnded) return

        if (!init) {
            stream.once('end', () => {
                addon.registerReadEnd(ctx)
            })

            init = true
        }

        stream.once('readable', function consume() {
            const buf = stream.read(read_size)
            if (buf) {
                addon.registerReadBuf(ctx, buf)
            } else {
                stream.once('readable', consume)
            }
        })
    })
    console.log(vips)
}

(async () => {
    await test()
})();