const fs = require('fs');

function test(){
    const stream = fs.createReadStream('/home/bongole/image/4k.jpg')
    let start = null
    let size = 0
    stream.on('data', (buf) => {
        if( start == null ) start = new Date()
        size += buf.length
    })

    /*
    stream.once('readable', function consume(){
        if( start == null ) start = new Date()

        let buf = null
        while(buf = stream.read(4096)){
            // while(buf = stream.read()){
            size += buf.length
        }
        stream.once('readable', consume)
    })
    */

    stream.on('finish', (buf) => {
        console.log('fin')
    })

    stream.on('close', (buf) => {
        let end = new Date()
        //console.log(end - start, size)
    })
}

for( let i = 0; i < 100; i++ ){
    test()
}
