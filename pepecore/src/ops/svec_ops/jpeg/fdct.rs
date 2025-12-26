// https://github.com/vstroebel/jpeg-encoder/blob/main/src/fdct.rs
/*
 * Ported from mozjpeg to rust
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2015, 2020, D. R. Commander.
 *
 * Conditions of distribution and use:
 * In plain English:
 *
 * 1. We don't promise that this software works.  (But if you find any bugs,
 *    please let us know!)
 * 2. You can use this software for whatever you want.  You don't have to pay us.
 * 3. You may not pretend that you wrote this software.  If you use it in a
 *    program, you must acknowledge somewhere in your documentation that
 *    you've used the IJG code.
 *
 * In legalese:
 *
 * The authors make NO WARRANTY or representation, either express or implied,
 * with respect to this software, its quality, accuracy, merchantability, or
 * fitness for a particular purpose.  This software is provided "AS IS", and you,
 * its user, assume the entire risk as to its quality and accuracy.
 *
 * This software is copyright (C) 1991-2020, Thomas G. Lane, Guido Vollbeding.
 * All Rights Reserved except as specified below.
 *
 * Permission is hereby granted to use, copy, modify, and distribute this
 * software (or portions thereof) for any purpose, without fee, subject to these
 * conditions:
 * (1) If any part of the source code for this software is distributed, then this
 * README file must be included, with this copyright and no-warranty notice
 * unaltered; and any additions, deletions, or changes to the original files
 * must be clearly indicated in accompanying documentation.
 * (2) If only executable code is distributed, then the accompanying
 * documentation must state that "this software is based in part on the work of
 * the Independent JPEG Group".
 * (3) Permission for use of this software is granted only if the user accepts
 * full responsibility for any undesirable consequences; the authors accept
 * NO LIABILITY for damages of any kind.
 *
 * These conditions apply to any software derived from or based on the IJG code,
 * not just to the unmodified library.  If you use our work, you ought to
 * acknowledge us.
 *
 * Permission is NOT granted for the use of any IJG author's name or company name
 * in advertising or publicity relating to this software or products derived from
 * it.  This software may be referred to only as "the Independent JPEG Group's
 * software".
 *
 * We specifically permit and encourage the use of this software as the basis of
 * commercial products, provided that all warranty or liability claims are
 * assumed by the product vendor.
 *
 * This file contains a slower but more accurate integer implementation of the
 * forward DCT (Discrete Cosine Transform).
 *
 * A 2-D DCT can be done by 1-D DCT on each row followed by 1-D DCT
 * on each column.  Direct algorithms are also available, but they are
 * much more complex and seem not to be any faster when reduced to code.
 *
 * This implementation is based on an algorithm described in
 *   C. Loeffler, A. Ligtenberg and G. Moschytz, "Practical Fast 1-D DCT
 *   Algorithms with 11 Multiplications", Proc. Int'l. Conf. on Acoustics,
 *   Speech, and Signal Processing 1989 (ICASSP '89), pp. 988-991.
 * The primary algorithm described there uses 11 multiplies and 29 adds.
 * We use their alternate method with 12 multiplies and 32 adds.
 * The advantage of this method is that no data path contains more than one
 * multiplication; this allows a very simple and accurate implementation in
 * scaled fixed-point arithmetic, with a minimal number of shifts.
 */

const CONST_BITS: i32 = 13;
const PASS1_BITS: i32 = 2;

const FIX_0_298631336: i32 = 2446;
const FIX_0_390180644: i32 = 3196;
const FIX_0_541196100: i32 = 4433;
const FIX_0_765366865: i32 = 6270;
const FIX_0_899976223: i32 = 7373;
const FIX_1_175875602: i32 = 9633;
const FIX_1_501321110: i32 = 12299;
const FIX_1_847759065: i32 = 15137;
const FIX_1_961570560: i32 = 16069;
const FIX_2_053119869: i32 = 16819;
const FIX_2_562915447: i32 = 20995;
const FIX_3_072711026: i32 = 25172;

const DCT_SIZE: usize = 8;

#[inline(always)]
fn descale(x: i32, n: i32) -> i32 {
    // right shift with rounding
    (x + (1 << (n - 1))) >> n
}

#[inline(always)]
fn into_el(v: i32) -> i16 {
    v as i16
}

#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
pub fn fdct(data: &mut [i16; 64]) {
    /* Pass 1: process rows. */
    /* Note results are scaled up by sqrt(8) compared to a true DCT; */
    /* furthermore, we scale the results by 2**PASS1_BITS. */

    let mut data2 = [0i32; 64];

    for y in 0..8 {
        let offset = y * 8;

        let tmp0 = i32::from(data[offset + 0]) + i32::from(data[offset + 7]);
        let tmp7 = i32::from(data[offset + 0]) - i32::from(data[offset + 7]);
        let tmp1 = i32::from(data[offset + 1]) + i32::from(data[offset + 6]);
        let tmp6 = i32::from(data[offset + 1]) - i32::from(data[offset + 6]);
        let tmp2 = i32::from(data[offset + 2]) + i32::from(data[offset + 5]);
        let tmp5 = i32::from(data[offset + 2]) - i32::from(data[offset + 5]);
        let tmp3 = i32::from(data[offset + 3]) + i32::from(data[offset + 4]);
        let tmp4 = i32::from(data[offset + 3]) - i32::from(data[offset + 4]);

        /* Even part per LL&M figure 1 --- note that published figure is faulty;
         * rotator "sqrt(2)*c1" should be "sqrt(2)*c6".
         */

        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;

        data2[offset + 0] = (tmp10 + tmp11) << PASS1_BITS;
        data2[offset + 4] = (tmp10 - tmp11) << PASS1_BITS;

        let z1 = (tmp12 + tmp13) * FIX_0_541196100;
        data2[offset + 2] = descale(z1 + (tmp13 * FIX_0_765366865), CONST_BITS - PASS1_BITS);
        data2[offset + 6] = descale(z1 + (tmp12 * -FIX_1_847759065), CONST_BITS - PASS1_BITS);

        /* Odd part per figure 8 --- note paper omits factor of sqrt(2).
         * cK represents cos(K*pi/16).
         * i0..i3 in the paper are tmp4..tmp7 here.
         */

        let z1 = tmp4 + tmp7;
        let z2 = tmp5 + tmp6;
        let z3 = tmp4 + tmp6;
        let z4 = tmp5 + tmp7;
        let z5 = (z3 + z4) * FIX_1_175875602; /* sqrt(2) * c3 */

        let tmp4 = tmp4 * FIX_0_298631336; /* sqrt(2) * (-c1+c3+c5-c7) */
        let tmp5 = tmp5 * FIX_2_053119869; /* sqrt(2) * ( c1+c3-c5+c7) */
        let tmp6 = tmp6 * FIX_3_072711026; /* sqrt(2) * ( c1+c3+c5-c7) */
        let tmp7 = tmp7 * FIX_1_501321110; /* sqrt(2) * ( c1+c3-c5-c7) */
        let z1 = z1 * -FIX_0_899976223; /* sqrt(2) * ( c7-c3) */
        let z2 = z2 * -FIX_2_562915447; /* sqrt(2) * (-c1-c3) */
        let z3 = z3 * -FIX_1_961570560; /* sqrt(2) * (-c3-c5) */
        let z4 = z4 * -FIX_0_390180644; /* sqrt(2) * ( c5-c3) */

        let z3 = z3 + z5;
        let z4 = z4 + z5;

        data2[offset + 7] = descale(tmp4 + z1 + z3, CONST_BITS - PASS1_BITS);
        data2[offset + 5] = descale(tmp5 + z2 + z4, CONST_BITS - PASS1_BITS);
        data2[offset + 3] = descale(tmp6 + z2 + z3, CONST_BITS - PASS1_BITS);
        data2[offset + 1] = descale(tmp7 + z1 + z4, CONST_BITS - PASS1_BITS);
    }

    /* Pass 2: process columns.
     * We remove the PASS1_BITS scaling, but leave the results scaled up
     * by an overall factor of 8.
     */

    for x in 0..8 {
        let tmp0 = data2[DCT_SIZE * 0 + x] + data2[DCT_SIZE * 7 + x];
        let tmp7 = data2[DCT_SIZE * 0 + x] - data2[DCT_SIZE * 7 + x];
        let tmp1 = data2[DCT_SIZE * 1 + x] + data2[DCT_SIZE * 6 + x];
        let tmp6 = data2[DCT_SIZE * 1 + x] - data2[DCT_SIZE * 6 + x];
        let tmp2 = data2[DCT_SIZE * 2 + x] + data2[DCT_SIZE * 5 + x];
        let tmp5 = data2[DCT_SIZE * 2 + x] - data2[DCT_SIZE * 5 + x];
        let tmp3 = data2[DCT_SIZE * 3 + x] + data2[DCT_SIZE * 4 + x];
        let tmp4 = data2[DCT_SIZE * 3 + x] - data2[DCT_SIZE * 4 + x];

        /* Even part per LL&M figure 1 --- note that published figure is faulty;
         * rotator "sqrt(2)*c1" should be "sqrt(2)*c6".
         */

        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;

        data[DCT_SIZE * 0 + x] = into_el(descale(tmp10 + tmp11, PASS1_BITS));
        data[DCT_SIZE * 4 + x] = into_el(descale(tmp10 - tmp11, PASS1_BITS));

        let z1 = (tmp12 + tmp13) * FIX_0_541196100;
        data[DCT_SIZE * 2 + x] = into_el(descale(z1 + tmp13 * FIX_0_765366865, CONST_BITS + PASS1_BITS));
        data[DCT_SIZE * 6 + x] = into_el(descale(z1 + tmp12 * -FIX_1_847759065, CONST_BITS + PASS1_BITS));

        /* Odd part per figure 8 --- note paper omits factor of sqrt(2).
         * cK represents cos(K*pi/16).
         * i0..i3 in the paper are tmp4..tmp7 here.
         */

        let z1 = tmp4 + tmp7;
        let z2 = tmp5 + tmp6;
        let z3 = tmp4 + tmp6;
        let z4 = tmp5 + tmp7;
        let z5 = (z3 + z4) * FIX_1_175875602; /* sqrt(2) * c3 */

        let tmp4 = tmp4 * FIX_0_298631336; /* sqrt(2) * (-c1+c3+c5-c7) */
        let tmp5 = tmp5 * FIX_2_053119869; /* sqrt(2) * ( c1+c3-c5+c7) */
        let tmp6 = tmp6 * FIX_3_072711026; /* sqrt(2) * ( c1+c3+c5-c7) */
        let tmp7 = tmp7 * FIX_1_501321110; /* sqrt(2) * ( c1+c3-c5-c7) */
        let z1 = z1 * -FIX_0_899976223; /* sqrt(2) * ( c7-c3) */
        let z2 = z2 * -FIX_2_562915447; /* sqrt(2) * (-c1-c3) */
        let z3 = z3 * -FIX_1_961570560; /* sqrt(2) * (-c3-c5) */
        let z4 = z4 * -FIX_0_390180644; /* sqrt(2) * ( c5-c3) */

        let z3 = z3 + z5;
        let z4 = z4 + z5;

        data[DCT_SIZE * 7 + x] = into_el(descale(tmp4 + z1 + z3, CONST_BITS + PASS1_BITS));
        data[DCT_SIZE * 5 + x] = into_el(descale(tmp5 + z2 + z4, CONST_BITS + PASS1_BITS));
        data[DCT_SIZE * 3 + x] = into_el(descale(tmp6 + z2 + z3, CONST_BITS + PASS1_BITS));
        data[DCT_SIZE * 1 + x] = into_el(descale(tmp7 + z1 + z4, CONST_BITS + PASS1_BITS));
    }
}

const SCALE_BITS: i32 = 512 + 65536 + (128 << 17);

#[inline(always)]
fn wa(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[inline(always)]
fn ws(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}

#[inline(always)]
fn wm(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}
pub fn idct_int_1x1(in_vector: &mut [i32; 64], mut out_vector: &mut [i16], stride: usize) {
    let coeff = ((wa(wa(in_vector[0], 4), 1024) >> 3).clamp(0, 255)) as i16;

    out_vector[..8].fill(coeff);
    for _ in 0..7 {
        out_vector = &mut out_vector[stride..];
        out_vector[..8].fill(coeff);
    }
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
fn f2f(x: f32) -> i32 {
    (x * 4096.0 + 0.5) as i32
}

#[inline]
fn fsh(x: i32) -> i32 {
    x << 12
}
#[inline]
fn clamp(a: i32) -> i16 {
    a.clamp(0, 255) as i16
}
pub fn idct_int(in_vector: &mut [i32; 64], out_vector: &mut [i16], stride: usize) {
    let mut pos = 0;
    let mut i = 0;

    if &in_vector[1..] == &[0_i32; 63] {
        return idct_int_1x1(in_vector, out_vector, stride);
    }

    // vertical pass
    for ptr in 0..8 {
        let p2 = in_vector[ptr + 16];
        let p3 = in_vector[ptr + 48];

        let p1 = wm(wa(p2, p3), 2217);

        let t2 = wa(p1, wm(p3, -7567));
        let t3 = wa(p1, wm(p2, 3135));

        let p2 = in_vector[ptr];
        let p3 = in_vector[32 + ptr];

        let t0 = fsh(wa(p2, p3));
        let t1 = fsh(ws(p2, p3));

        let x0 = wa(wa(t0, t3), 512);
        let x3 = wa(ws(t0, t3), 512);
        let x1 = wa(wa(t1, t2), 512);
        let x2 = wa(ws(t1, t2), 512);

        let mut t0 = in_vector[ptr + 56];
        let mut t1 = in_vector[ptr + 40];
        let mut t2 = in_vector[ptr + 24];
        let mut t3 = in_vector[ptr + 8];

        let p3 = wa(t0, t2);
        let p4 = wa(t1, t3);
        let p1 = wa(t0, t3);
        let p2 = wa(t1, t2);
        let p5 = wm(wa(p3, p4), 4816);

        t0 = wm(t0, 1223);
        t1 = wm(t1, 8410);
        t2 = wm(t2, 12586);
        t3 = wm(t3, 6149);

        let p1 = wa(p5, wm(p1, -3685));
        let p2 = wa(p5, wm(p2, -10497));
        let p3 = wm(p3, -8034);
        let p4 = wm(p4, -1597);

        t3 = wa(t3, wa(p1, p4));
        t2 = wa(t2, wa(p2, p3));
        t1 = wa(t1, wa(p2, p4));
        t0 = wa(t0, wa(p1, p3));

        in_vector[ptr] = ws(wa(x0, t3), 0) >> 10;
        in_vector[ptr + 8] = ws(wa(x1, t2), 0) >> 10;
        in_vector[ptr + 16] = ws(wa(x2, t1), 0) >> 10;
        in_vector[ptr + 24] = ws(wa(x3, t0), 0) >> 10;
        in_vector[ptr + 32] = ws(ws(x3, t0), 0) >> 10;
        in_vector[ptr + 40] = ws(ws(x2, t1), 0) >> 10;
        in_vector[ptr + 48] = ws(ws(x1, t2), 0) >> 10;
        in_vector[ptr + 56] = ws(ws(x0, t3), 0) >> 10;
    }

    // horizontal pass
    while i < 64 {
        let p2 = in_vector[i + 2];
        let p3 = in_vector[i + 6];

        let p1 = wm(wa(p2, p3), 2217);
        let t2 = wa(p1, wm(p3, -7567));
        let t3 = wa(p1, wm(p2, 3135));

        let p2 = in_vector[i];
        let p3 = in_vector[i + 4];

        let t0 = fsh(wa(p2, p3));
        let t1 = fsh(ws(p2, p3));

        let x0 = wa(wa(t0, t3), SCALE_BITS);
        let x3 = wa(ws(t0, t3), SCALE_BITS);
        let x1 = wa(wa(t1, t2), SCALE_BITS);
        let x2 = wa(ws(t1, t2), SCALE_BITS);

        let mut t0 = in_vector[i + 7];
        let mut t1 = in_vector[i + 5];
        let mut t2 = in_vector[i + 3];
        let mut t3 = in_vector[i + 1];

        let p3 = wa(t0, t2);
        let p4 = wa(t1, t3);
        let p1 = wa(t0, t3);
        let p2 = wa(t1, t2);
        let p5 = wm(wa(p3, p4), f2f(1.175875602));

        t0 = wm(t0, 1223);
        t1 = wm(t1, 8410);
        t2 = wm(t2, 12586);
        t3 = wm(t3, 6149);

        let p1 = wa(p5, wm(p1, -3685));
        let p2 = wa(p5, wm(p2, -10497));
        let p3 = wm(p3, -8034);
        let p4 = wm(p4, -1597);

        t3 = wa(t3, wa(p1, p4));
        t2 = wa(t2, wa(p2, p3));
        t1 = wa(t1, wa(p2, p4));
        t0 = wa(t0, wa(p1, p3));

        let out: &mut [i16; 8] = out_vector.get_mut(pos..pos + 8).unwrap().try_into().unwrap();

        out[0] = clamp(wa(x0, t3) >> 17);
        out[1] = clamp(wa(x1, t2) >> 17);
        out[2] = clamp(wa(x2, t1) >> 17);
        out[3] = clamp(wa(x3, t0) >> 17);
        out[4] = clamp(ws(x3, t0) >> 17);
        out[5] = clamp(ws(x2, t1) >> 17);
        out[6] = clamp(ws(x1, t2) >> 17);
        out[7] = clamp(ws(x0, t3) >> 17);

        i += 8;
        pos += stride;
    }
}
