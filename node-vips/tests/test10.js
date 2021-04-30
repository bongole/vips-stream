const addon = require('./index.js');
const fs = require('fs');

function sleep(t){
    return new Promise((res) => setTimeout(res, t))
}

async function test(){
    const stream = fs.createReadStream('/home/bongole/image/4k.jpg', { highWaterMark: 64 * 1024 })
    //const buflist = new addon.BufferList(4 * 1024 * 1024);

    let vips = await new Promise((res, rej) => {
        let res_wrap = (_err, vips) => res(vips);
        const bufList = new addon.BufferList( 128 * 1024 );
        addon.createVipsImage(res_wrap, rej, bufList, () => {
            stream.on('close', () => {
                bufList.close()
            })

            stream.on('data', (buf) => {
                let r = bufList.push(buf)
                if( !r ) {
                    //console.log('pause')
                    stream.pause()
                }
            })
        }, () => { 
            //console.log('resume')
            stream.resume()
        });
    });

    console.log(vips)

    await new Promise((res, rej) => {
        const write_stream = fs.createWriteStream('/dev/null')

        const res_wrap = (_err, v) => {
            write_stream.end(() => {
                res(v);
            });
        };

        addon.writeVipsImage(vips, ".jpg", res_wrap, rej, async (err, ctx, buf, mystruct) => {
            if (!write_stream.writable) {
                addon.registerWriteSize(ctx, -1)
                return
            }

            let r = write_stream.write(buf)
            if (r) {
                addon.registerWriteSize(ctx, buf.length)
            } else {
                console.log('before drain ')
                let r = await Promise.race([new Promise((r) => write_stream.once('drain', () => r('drain'))), sleep(1000)])
                if (r === 'drain') {
                    addon.registerWriteSize(ctx, buf.length)
                } else {
                    console.log('after drain not writable ')
                    addon.registerWriteSize(ctx, -1)
                }
            }
        });
    });


}

for( let i = 0; i < 100; i++ ){
    test()
}
