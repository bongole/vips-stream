const addon = require('./index.js')
const fs = require('fs');

async function test(idx) {
    const read_stream = fs.createReadStream("/home/bongole/image/4k.jpg");

    let vips = await new Promise((res, rej) => {
        let init = false;
        const res_wrap = (_err, v) => res(v);
        /*
        const fd = fs.openSync("/home/bongole/image/4k.jpg", 'r')
        addon.createVipsImage(res_wrap, rej, (err, ctx, read_size) => {
            let buf = Buffer.alloc(parseInt(read_size))
            let r = fs.readSync(fd, buf)
            addon.registerReadBuf(ctx, buf)
        })
        */

        let count = 0
        addon.createVipsImage(res_wrap, rej, async (err, ctx, read_size) => {
            /*
            if (read_stream.readableEnded) return

            if (!init) {
                read_stream.once('end', () => {
                    //addon.registerReadEnd(ctx)
                    console.log(count)
                    read_stream.close()
                })

                read_stream.once('error', (e) => {
                    //addon.registerReadEnd(ctx)
                    rej(e)
                })

                init = true
            }
            */

            let buf = await new Promise((res) => read_stream.once('readable', () => {
                const b = read_stream.read(read_size)
                res(b)
            }))

            if( buf ){
                count += buf.length
                addon.registerReadBuf(ctx, buf)
            }
        })
    })

    console.log('vips = ', vips)

    const write_stream = fs.createWriteStream(`/tmp/test/thumb${idx}.jpg`);
    let r = await new Promise((res, rej) => {
        const res_wrap = (_err, v) => {
            write_stream.end(() => {
                console.log('write end js')
                write_stream.close();
                res(v);
            })
        }

        addon.writeVipsImage(vips, ".jpg", res_wrap, rej, async (err, ctx, buf, mystruct) => {
            if (!write_stream.writable) return

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
    global.gc()
    console.log('start ' + process.pid)
    //await new Promise((r) => setTimeout(r, 10000))
    let proms = [];
    for (let i = 0; i < 1; i++) {
        proms.push(test(i))
    }

    await Promise.all(proms)
    console.log('end')
    addon.shutdown()
    global.gc()
    await new Promise((r) => setTimeout(r, 10000))
})();
