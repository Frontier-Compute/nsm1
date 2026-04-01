#!/usr/bin/env node
/**
 * Lightweight Zcash memo format detector. Zero dependencies.
 * Identifies: ZAP1, ZIP 302 TVLV, plain text, binary, empty.
 *
 * Usage:
 *   node memo_decode.js <hex_memo>
 *   echo "5a4150313a..." | node memo_decode.js
 *
 * Drop this into any JS project. No npm install needed.
 */

function decodeMemo(hexOrBytes) {
  const bytes = typeof hexOrBytes === 'string'
    ? Buffer.from(hexOrBytes, 'hex')
    : hexOrBytes;

  if (bytes.length === 0) return { format: 'empty' };

  // Check for all-zero (empty memo padded)
  if (bytes.every(b => b === 0)) return { format: 'empty' };

  // ZAP1 / NSM1 structured memo: "ZAP1:" or "NSM1:" prefix
  const ascii = bytes.slice(0, 5).toString('ascii');
  if (ascii === 'ZAP1:' || ascii === 'NSM1:') {
    const full = bytes.toString('ascii').replace(/\0+$/, '');
    const parts = full.split(':');
    if (parts.length >= 3) {
      const protocol = parts[0];
      const typeByte = parseInt(parts[1], 16);
      const payload = parts.slice(2).join(':');
      const typeNames = {
        0x01: 'PROGRAM_ENTRY', 0x02: 'OWNERSHIP_ATTEST', 0x03: 'CONTRACT_ANCHOR',
        0x04: 'DEPLOYMENT', 0x05: 'HOSTING_PAYMENT', 0x06: 'SHIELD_RENEWAL',
        0x07: 'TRANSFER', 0x08: 'EXIT', 0x09: 'MERKLE_ROOT',
        0x0A: 'STAKING_DEPOSIT', 0x0B: 'STAKING_WITHDRAW', 0x0C: 'STAKING_REWARD',
      };
      return {
        format: 'zap1',
        protocol,
        event_type: typeByte,
        event_label: typeNames[typeByte] || `UNKNOWN_0x${typeByte.toString(16).padStart(2, '0')}`,
        payload,
      };
    }
  }

  // ZIP 302 TVLV: starts with 0xF7
  if (bytes[0] === 0xF7) {
    return { format: 'zip302_tvlv', first_byte: '0xf7', length: bytes.length };
  }

  // Plain text: check if valid UTF-8 printable
  const trimmed = bytes.slice(0, bytes.indexOf(0) === -1 ? bytes.length : bytes.indexOf(0));
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(trimmed);
    if (text.length > 0 && /^[\x20-\x7E\n\r\t]+$/.test(text)) {
      return { format: 'text', content: text };
    }
  } catch {}

  return { format: 'binary', length: bytes.length, first_byte: `0x${bytes[0].toString(16).padStart(2, '0')}` };
}

// CLI
if (typeof process !== 'undefined' && process.argv) {
  const input = process.argv[2] || '';
  if (input) {
    console.log(JSON.stringify(decodeMemo(input), null, 2));
  } else {
    // Read from stdin
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', chunk => data += chunk);
    process.stdin.on('end', () => {
      if (data.trim()) console.log(JSON.stringify(decodeMemo(data.trim()), null, 2));
      else console.log('Usage: node memo_decode.js <hex_memo>');
    });
  }
}

// Export for use as module
if (typeof module !== 'undefined') module.exports = { decodeMemo };
