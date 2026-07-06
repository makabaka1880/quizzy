# Encoding & Decoding Specification v1.1

> **Audience:** Graders, tooling authors, and anyone who needs to recover student answers and answer keys from an encoded Quizzy blob.

## Overview

When a student completes a quiz, Quizzy produces an **encoded blob** — a single opaque string carrying the student's answers, per-question timing, correctness, and the answer key. The encoding uses **two independent encryption blocks** with different keys so that:

- The **student** can decode their own answers (session block — key derived from `studentId`).
- Only the **teacher** (who holds the `quizKey`) can decode the answer key.

Each block is independently encrypted, salted, and checksummed.

---

## Output Format (what the student copies)

```
<header>.<session_block>.<answer_key_block>
```

| Segment            | Description                                                          |
|--------------------|----------------------------------------------------------------------|
| `header`           | Base64 of `{"sid":"<studentId>","ts":<unix_millis>}`                 |
| `.`                | Literal dot separator                                                |
| `session_block`    | `<checksum_8_hex><interleaved>` — encrypted with student-key         |
| `.`                | Literal dot separator                                                |
| `answer_key_block` | `<checksum_8_hex><interleaved>` — encrypted with quiz-key            |

### Block Breakdown (both blocks use the same internal structure)

| Offset | Length | Content                                          |
|--------|--------|--------------------------------------------------|
| 0      | 8      | First 8 hex characters of SHA-256(interleaved)   |
| 8      | rest   | Interleaved payload (ciphertext_base64 + salt)   |

---

## Cipher Primitives

### Key Derivation

```
derive_key(seed: string, timestamp: u64) → [u8; 32]
    = SHA-256(seed + ":" + timestamp)
```

Concatenation uses `:` as separator. The hash is 32 bytes.

Two keys are derived:

| Key              | Seed           | Purpose                                  |
|------------------|----------------|------------------------------------------|
| `student_key`    | `studentId`    | Encrypt/decrypt the session block        |
| `quiz_cipher_key`| `quizKey`      | Encrypt/decrypt the answer-key block     |

### XOR Cipher

The cipher operates on **bytes**, not characters:

```
for each byte B at index i:
    encrypted[i] = B XOR key[i mod 32]
```

XOR is its own inverse — encryption and decryption are the same operation.

### Interleave

Two equal-length strings `A` and `B` are merged character-by-character:

```
result = ""
for i in 0..len(A)-1:
    result += A[i]
    result += B[i]
```

Both strings **must** have the same length. Example: interleaving `"abcdef"` with `"123456"` → `"a1b2c3d4e5f6"`.

### De-interleave

Reverse: characters at even indices go to `A`, odd indices go to `B`:

```
A = ""
B = ""
for i in 0..len(combined)-1:
    if i mod 2 == 0: A += combined[i]
    else:            B += combined[i]
```

### Salt

```
generate_salt(length: usize) → String
    = length random characters from alphabet "A-Za-z0-9+/="
```

Cryptographically random (uses OS CSPRNG).

### Checksum

```
sha256_hex(input: &str) → String
    = hex-encoded SHA-256 digest (64 lowercase hex chars)
```

---

## Encoding Pipeline (Block Level)

Each block is encoded independently with its own key:

### Inputs

- `payload`: the object to encrypt (JSON-serializable)
- `key`: a `[u8; 32]` from `derive_key()`

### Step 1 — JSON Serialize

```
json = JSON.stringify(payload)    // compact, no whitespace
```

### Step 2 — XOR Encrypt

```
json_bytes = utf8_encode(json)
ciphered = XOR_each_byte(json_bytes, key)
```

### Step 3 — Base64 Encode

```
b64 = standard_base64(ciphered)
```

Standard Base64 (RFC 4648, padding with `=`).

### Step 4 — Salt & Interleave

```
salt = generate_salt(b64.len())   // salt is same length as b64
interleaved = interleave(b64, salt)
```

**Critical:** salt length **equals** base64 length (not `max(32, len/4)`). This ensures the interleave assertion (`a.len() == b.len()`) holds.

### Step 5 — Checksum

```
checksum_full = sha256_hex(interleaved)
checksum = checksum_full[0..8]    // first 8 hex chars
```

### Step 6 — Block Assembly

```
block = checksum + interleaved
```

---

## Full Encoding Pipeline

### Inputs

- `student_id`: the student's ID string
- `timestamp`: Unix milliseconds (e.g. `Date.now()`)
- `session`: `SessionState` object (student answers, timing, grading)
- `problems`: `Problem[]` array (original problem set)
- `quiz_key`: the instructor's secret key string

### Step 1 — Build Answer Key

From the original `problems`, build a `BTreeMap<u64, AnswerKeyEntry>`:

```
for each problem p:
  key[p.id] = match p.variant.kind:
    "mcq"  → { kind:"mcq",  correctIdx: p.variant.correctIdx, points: p.points }
    "mmcq" → { kind:"mmcq", correctIdxs: p.variant.correctIdxs, points: p.points }
    "frq"  → { kind:"frq",  expectedPattern: p.variant.expectedPattern,
                             strategy: p.variant.strategy, points: p.points }
    "ffrq" → { kind:"ffrq", points: p.points, grading: "human" }
```

### Step 2 — Derive Keys

```
student_key     = derive_key(student_id, timestamp)
quiz_cipher_key = derive_key(quiz_key, timestamp)
```

### Step 3 — Encode Session Block

```
session_payload = { session: <SessionState> }
session_block = encode_block(session_payload, student_key)
```

### Step 4 — Encode Answer-Key Block

```
answer_key_payload = { answerKey: <answer key from Step 1> }
answer_key_block = encode_block(answer_key_payload, quiz_cipher_key)
```

### Step 5 — Build Header

```
header = base64(json({"sid": student_id, "ts": timestamp}))
```

### Step 6 — Final Assembly

```
output = header + "." + session_block + "." + answer_key_block
```

---

## Decoding Pipeline

### decode_session(encoded) → SessionState

Decodes only the session block — does **not** require `quiz_key`.

1. Split on `.` — require at least 2 parts
2. Base64-decode `parts[0]` → parse JSON → extract `sid` and `ts`
3. `student_key = derive_key(sid, ts)`
4. `block = parts[1]`
5. Decode the block with `student_key` → parse `{ session: SessionState }`
6. Return `SessionState`

### decode_answer_key(encoded, quiz_key) → AnswerKey

Decodes only the answer-key block — requires `quiz_key`.

1. Split on `.` — require at least 3 parts
2. Base64-decode `parts[0]` → parse JSON → extract `ts`
3. `quiz_cipher_key = derive_key(quiz_key, ts)`
4. `block = parts[2]`
5. Decode the block with `quiz_cipher_key` → parse `{ answerKey: AnswerKey }`
6. Return `AnswerKey`

### decode_all(encoded, quiz_key) → DecodedResult

Combines both:

```
session    = decode_session(encoded)
answer_key = decode_answer_key(encoded, quiz_key)
return { session, answerKey: answer_key }
```

### Block Decode Algorithm

```
decode_block(block_str, key) → object:
    1. checksum_prefix = block_str[0..8]
    2. interleaved     = block_str[8..]
    3. computed = sha256_hex(interleaved)[0..8]
    4. assert computed == checksum_prefix   // fail if mismatch
    5. (b64, _salt) = deinterleave(interleaved)
    6. ciphered = base64_decode(b64)
    7. plain_bytes = XOR_each_byte(ciphered, key)   // decrypt = encrypt
    8. json = utf8_decode(plain_bytes)
    9. return JSON.parse(json)
```

---

## Data Structures (Post-Decode)

### `SessionState`

```jsonc
{
  "studentId": "jsmith42",
  "timestamp": 1711234567890,
  "entries": [ /* GradingEntry[] */ ]
}
```

### `GradingEntry`

| Field            | Type                        | Description                                   |
|------------------|-----------------------------|-----------------------------------------------|
| `problemId`      | `number`                    | Which problem this entry corresponds to       |
| `isCorrect`      | `boolean \| null`           | `null` = pending human grading (FFRQ)         |
| `gradingStatus`  | `"auto-graded" \| "pending-human"` | Grading disposition                   |
| `answer`         | `Answer`                    | Student's answer (tagged union, see below)    |
| `timeSpentMs`    | `number`                    | Milliseconds spent on this question           |
| `pointsAwarded`  | `number`                    | Always 0 for FFRQ until human overrides       |

### `Answer` (Tagged Union)

| `kind`   | `value` type  | Example                              |
|----------|---------------|--------------------------------------|
| `"mcq"`  | `number`      | `{"kind":"mcq","value":2}`           |
| `"mmcq"` | `number[]`    | `{"kind":"mmcq","value":[0,3]}`      |
| `"frq"`  | `string`      | `{"kind":"frq","value":"O(log n)"}`  |
| `"ffrq"` | `string`      | `{"kind":"ffrq","value":"In my opinion..."}` |

### `AnswerKey` (Map of `problemId → AnswerKeyEntry`)

| `kind`   | Extra fields                                                     |
|----------|------------------------------------------------------------------|
| `"mcq"`  | `correctIdx: number`, `points: number`                            |
| `"mmcq"` | `correctIdxs: number[]`, `points: number`                         |
| `"frq"`  | `expectedPattern: string`, `strategy: "exact"\|"contains"\|"regex"`, `points: number` |
| `"ffrq"` | `points: number`, `grading: "human"`                              |

---

## Reference Decoder (Python)

```python
import json
import hashlib
import base64

def decode_session(encoded: str) -> dict:
    """Decode the student session block. No quiz_key needed."""
    parts = encoded.split('.')
    if len(parts) < 2:
        raise ValueError('Invalid format: expected at least header.session_block')

    header = json.loads(base64.b64decode(parts[0]).decode('utf-8'))
    sid, ts = header['sid'], header['ts']
    student_key = derive_key(sid, ts)
    return decode_block(parts[1], student_key)['session']


def decode_answer_key(encoded: str, quiz_key: str) -> dict:
    """Decode the answer key block. Requires quiz_key."""
    parts = encoded.split('.')
    if len(parts) < 3:
        raise ValueError('Invalid format: expected header.session_block.answer_key_block')

    header = json.loads(base64.b64decode(parts[0]).decode('utf-8'))
    ts = header['ts']
    quiz_cipher_key = derive_key(quiz_key, ts)
    return decode_block(parts[2], quiz_cipher_key)['answerKey']


def decode_all(encoded: str, quiz_key: str) -> dict:
    return {
        'session': decode_session(encoded),
        'answerKey': decode_answer_key(encoded, quiz_key),
    }


def decode_block(block_str: str, key: bytes) -> dict:
    if len(block_str) < 8:
        raise ValueError('Block too short')

    checksum_prefix = block_str[:8]
    interleaved = block_str[8:]

    # Verify checksum
    computed = hashlib.sha256(interleaved.encode('utf-8')).hexdigest()[:8]
    if computed != checksum_prefix:
        raise ValueError('Checksum mismatch — data may be corrupted')

    # De-interleave
    b64 = ''.join(interleaved[i] for i in range(0, len(interleaved), 2))

    # Base64 decode
    ciphered = base64.b64decode(b64)

    # XOR decrypt
    plain = xor_cipher(ciphered, key)

    return json.loads(plain.decode('utf-8'))


def derive_key(seed: str, timestamp: int) -> bytes:
    payload = f'{seed}:{timestamp}'
    return hashlib.sha256(payload.encode('utf-8')).digest()


def xor_cipher(data: bytes, key: bytes) -> bytes:
    return bytes(data[i] ^ key[i % len(key)] for i in range(len(data)))
```

## Reference Decoder (JavaScript)

```js
async function decodeSession(encoded) {
  const parts = encoded.split('.');
  if (parts.length < 2) throw new Error('Invalid format');

  const header = JSON.parse(atob(parts[0]));
  const studentKey = await deriveKey(header.sid, header.ts);
  const payload = decodeBlock(parts[1], studentKey);
  return payload.session;
}

async function decodeAnswerKey(encoded, quizKey) {
  const parts = encoded.split('.');
  if (parts.length < 3) throw new Error('Invalid format');

  const header = JSON.parse(atob(parts[0]));
  const cipherKey = await deriveKey(quizKey, header.ts);
  const payload = decodeBlock(parts[2], cipherKey);
  return payload.answerKey;
}

function decodeBlock(blockStr, key) {
  if (blockStr.length < 8) throw new Error('Block too short');

  const checksumPrefix = blockStr.slice(0, 8);
  const interleaved = blockStr.slice(8);

  return sha256Hex(interleaved).then(hash => {
    if (hash.slice(0, 8) !== checksumPrefix) {
      throw new Error('Checksum mismatch');
    }

    let b64 = '';
    for (let i = 0; i < interleaved.length; i += 2) b64 += interleaved[i];

    const ciphered = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
    const plain = xorCipher(ciphered, key);
    return JSON.parse(new TextDecoder().decode(plain));
  });
}

async function deriveKey(seed, timestamp) {
  const payload = `${seed}:${timestamp}`;
  const data = new TextEncoder().encode(payload);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return new Uint8Array(hash);
}

function xorCipher(data, key) {
  return data.map((b, i) => b ^ key[i % key.length]);
}

async function sha256Hex(input) {
  const data = new TextEncoder().encode(input);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return Array.from(new Uint8Array(hash))
    .map(b => b.toString(16).padStart(2, '0')).join('');
}
```

## Rust Reference Implementation

The canonical implementation is `quizzy-core` at `QuizzyCore/src/encode.rs` and `QuizzyCore/src/crypto.rs`.

Key entry points:
- `encode_full(student_id, timestamp, session, problems, quiz_key) → String`
- `decode_session(encoded) → Result<SessionState, String>`
- `decode_answer_key(encoded, quiz_key) → Result<AnswerKey, String>`
- `decode_all(encoded, quiz_key) → Result<DecodedResult, String>`

---

## Security Notes

- **Dual-key design.** The session block is encrypted with `derive_key(studentId, timestamp)` — the student can decode it. The answer-key block is encrypted with `derive_key(quizKey, timestamp)` — only the teacher (who holds `quizKey`) can decode it. The student never sees `quizKey` in recoverable form.
- **Checksum detects tampering.** Each block carries an 8-char SHA-256 checksum prefix. Any modification to the interleaved payload will cause a checksum mismatch on decode.
- **Timestamps act as nonces.** Two quiz submissions from the same student produce different blobs (different timestamp → different keys → different ciphertext and salt), even if answers are identical.
