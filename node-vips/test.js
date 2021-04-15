const addon = require('./index.js')
const fs = require('fs');

async function test(idx) {
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");

    let vips = await new Promise((res, rej) => {
        let init = false;
        const res_wrap = (_err, v) => res(v);
        addon.createVipsImage(res_wrap, rej, (err, ctx, read_size) => {
            if (read_stream.readableEnded) return

            if (!init) {
                read_stream.once('end', () => {
                    addon.registerReadEnd(ctx)
                    addon.showReadCtxRefCount(ctx)
                    console.log('read end js')
                    read_stream.close()
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
    addon.showVipsImageRefCount(vips)
    setInterval(() => {
        addon.showVipsImageRefCount(vips)
        console.log('gc')
        global.gc()
    }, 1000);
    */

    const write_stream = fs.createWriteStream(`/tmp/test/thumb${idx}.jpg`);
    let r = await new Promise((res, rej) => {
        const res_wrap = (_err, v) => { write_stream.end(() => { 
            console.log('write end js')
            write_stream.close();
            res(v); 
        }) }

        addon.writeVipsImage(vips, ".jpg", res_wrap, rej, async (err, ctx, buf) => {
            if (!write_stream.writable ) return

            let r = write_stream.write(buf)
            if (!r) {
                await new Promise((r) => write_stream.once('drain', r))
                console.log('drain')
            }

            addon.registerWriteSize(ctx, buf.length)
        });
    });
    console.log(r)
}

(async () => {
    let proms = [];
    for (let i = 0; i < 100; i++) {
        test(i)
    }

})();
