const addon = require('./index.js')
const fs = require('fs');

async function test(idx) {
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");

    let vips = await new Promise((res, rej) => {
        let init = false;
        const res_wrap = (_err, v) => res(v);
        addon.createVipsImage(res_wrap, rej, (err, ctx, read_size) => {
            if (read_stream.readableEnded ) return

            if (!init) {
                read_stream.once('end', () => {
                    addon.registerReadEnd(ctx)
                })

                read_stream.once('error', (e) => {
                    addon.registerReadEnd(ctx)
                    rej(e)
                })

                init = true
            }

            read_stream.once('readable', function consume() {
                const buf = read_stream.read(read_size)
                if (buf) {
                    addon.registerReadBuf(ctx, buf)
                } else {
                    read_stream.once('readable', consume)
                }
            })
        })
    })

    console.log(vips)

    /*
    const write_stream = fs.createWriteStream(`/tmp/test/thumb${idx}.jpg`);
    addon.writeVipsImage(vips, ".jpg", async (err, ctx, buf) => {
        console.log('write')
        if( !write_stream.writable ) return
        console.log(buf)

        let r = write_stream.write(buf)
        console.log(`r = ${r}`)
        if( !r ) {
            await new Promise((r) => write_stream.once('drain', r))
            console.log('drain')
        }

        console.log('wrote ' + buf.length)
        addon.registerWriteSize(ctx, buf.length)
    });
    */
}

(async () => {
    let proms = [];
    for (let i = 0; i < 10; i++) {
        proms.push(test(i))
    }

    await Promise.all(proms)
})();
