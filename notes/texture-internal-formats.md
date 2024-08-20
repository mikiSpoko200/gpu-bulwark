# Internal format

* If the *internalformat* of the texture is signed or unsigned integer, components are clamped to $[2^n - 1; 2^{n - 1} - 1]$ or $[0; 2^n - 1]$,respectively, where n is the number of bits per component.

* For color component groups, if the *internalformat* of the texture is signed or unsigned normalized fixed-point:
    – If the type of the data is a floating-point type (as defined in table 8.2), it is clamped to $[-1; 1]$ or $[0; 1]$, respectively.
    – Otherwise, it is clamped to to [ 2n 1 2n 1 1] or [02n 1], respec
tively, where n is the number of bits in the normalized representation.
 • For depth component groups, the depth value is clamped to [0; 1].
 • Stencil index values are masked by 2n 1, where n is the number of stencil
 bits in the internal format resolution (see below).
 • Otherwise, values are not modified

| *type* Parameter Token Name    | Corresponding GL Data Type | Special Interpretation | Floating Point |
| ------------------------------ | -------------------------- | ---------------------- | -------------- |
| UNSIGNED_BYTE                  | ubyte                      | No                     | No             |
| BYTE                           | byte                       | No                     | No             |
| UNSIGNED_SHORT                 | ushort                     | No                     | No             |
| SHORT                          | short                      | No                     | No             |
| UNSIGNED_INT                   | uint                       | No                     | No             |
| INT                            | int                        | No                     | No             |
| HALF_FLOAT                     | half                       | No                     | Yes            |
| FLOAT                          | float                      | No                     | Yes            |
| UNSIGNED_BYTE_3_3_2            | ubyte                      | Yes                    | No             |
| UNSIGNED_BYTE_2_3_3_REV        | ubyte                      | Yes                    | No             |
| UNSIGNED_SHORT_5_6_5           | ushort                     | Yes                    | No             |
| UNSIGNED_SHORT_5_6_5_REV       | ushort                     | Yes                    | No             |
| UNSIGNED_SHORT_4_4_4_4         | ushort                     | Yes                    | No             |
| UNSIGNED_SHORT_4_4_4_4_REV     | ushort                     | Yes                    | No             |
| UNSIGNED_SHORT_5_5_5_1         | ushort                     | Yes                    | No             |
| UNSIGNED_SHORT_1_5_5_5_REV     | ushort                     | Yes                    | No             |
| UNSIGNED_INT_8_8_8_8           | uint                       | Yes                    | No             |
| UNSIGNED_INT_8_8_8_8_REV       | uint                       | Yes                    | No             |
| UNSIGNED_INT_10_10_10_2        | uint                       | Yes                    | No             |
| UNSIGNED_INT_2_10_10_10_REV    | uint                       | Yes                    | No             |
| UNSIGNED_INT_24_8              | uint                       | Yes                    | No             |
| UNSIGNED_INT_10F_11F_11F_REV   | uint                       | Yes                    | Yes            |
| UNSIGNED_INT_5_9_9_9_REV       | uint                       | Yes                    | Yes            |
| FLOAT_32_UNSIGNED_INT_24_8_REV | n /a                       | Yes                    | N              |