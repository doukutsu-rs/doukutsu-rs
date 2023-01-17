// <reference types="node" />

const readline = require('readline');
const childProcess = require('child_process');

const rl = readline.createInterface({
    input: process.stdin,
    terminal: false
});

let textStart = 0;
const textStartRegex = /__text_start = 0x([0-9a-f]+)/i;
let symbolize = false;

if (process.argv.length <= 2) {
    console.error('Usage: node symbolize.js <path to ELF file>');
    process.exit(1);
}

const elfPath = process.argv[2];

rl.on('line', (line) => {
    if (textStart === 0) {
        const match = textStartRegex.exec(line);
        if (match) {
            textStart = parseInt(match[1], 16);
        }
    }

    if (line.includes("stack backtrace:")) {
        symbolize = true;
    }

    if (symbolize) {
        const match = /0x([0-9a-f]+) - \<unknown\>/.exec(line);
        if (match) {
            const addr = parseInt(match[1], 16);
            const relative = addr - textStart;
            // run addr2line on the address
            const addr2line = childProcess.spawnSync('addr2line', ['-e', elfPath, '-j', '.text', '-f', '-C', '0x' + relative.toString(16)]);
            if (addr2line.status === 0) {
                const output = addr2line.stdout.toString();
                const lines = output.split('\n');
                const [func, file] = lines;
                line = line.replace(match[0], `0x${addr.toString(16)} - ${func} (${file})`);
            }
        }
    }

    console.log(line);
});