use super::addressing_modes::EffectiveAddress;
use super::registers::Registers;
use crate::memory::Memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // Load and Store
    LDA, // Load Accumulator
    LDX, // Load X Register
    LDY, // Load Y Register
    STA, // Store Accumulator
    STX, // Store X Register
    STY, // Store Y Register
    STZ, // Store Zero
    TAX, // Transfer Accumulator to X
    TAY, // Transfer Accumulator to Y
    TXA, // Transfer X to Accumulator
    TYA, // Transfer Y to Accumulator
    TSX, // Transfer Stack Pointer to X
    TXS, // Transfer X to Stack Pointer
    TXY, // Transfer X to Y
    TYX, // Transfer Y to X
    TCD, // TAD – Transfer 16-bit Accumulator (C) to Direct Page Register
    TDC, // TDA – Transfer Direct Page Register to 16-bit Accumulator (C)
    TCS, // TAS – Transfer 16-bit Accumulator (C) to Stack Pointer
    TSC, // TSA – Transfer Stack Pointer to 16-bit Accumulator (C)

    // Arithmetic and Logic
    ADC, // Add with Carry
    SBC, // Subtract with Carry/Borrow
    AND, // Logical AND
    ORA, // Logical OR
    EOR, // Exclusive OR
    CMP, // Compare Accumulator
    CPX, // Compare X Register
    CPY, // Compare Y Register
    INC, // Increment
    INX, // Increment X
    INY, // Increment Y
    DEC, // Decrement
    DEX, // Decrement X
    DEY, // Decrement Y

    // Shift and Rotate
    ASL, // Arithmetic Shift Left
    LSR, // Logical Shift Right
    ROL, // Rotate Left
    ROR, // Rotate Right
    BIT, // Bit Test
    TRB, // Test and Reset Bits
    TSB, // Test and Set Bits

    // Branches
    BCC, // BLT – Branch if Carry Clear
    BCS, // BGE – Branch if Carry Set
    BEQ, // Branch if Equal
    BNE, // Branch if Not Equal
    BMI, // Branch if Minus
    BPL, // Branch if Plus
    BVC, // Branch if Overflow Clear
    BVS, // Branch if Overflow Set
    BRA, // Branch Always
    BRL, // Branch Relative Long

    // Jumps and Subroutines
    JMP, // Jump
    JML, // Jump Long
    JSR, // Jump to Subroutine
    JSL, // Jump to Subroutine Long
    RTS, // Return from Subroutine
    RTL, // Return from Subroutine Long
    RTI, // Return from Interrupt

    // Stack
    PHA, // Push Accumulator
    PLA, // Pull Accumulator
    PHX, // Push X Register
    PLX, // Pull X Register
    PHY, // Push Y Register
    PLY, // Pull Y Register

    PHP, // Push Processor Status
    PLP, // Pull Processor Status
    PHD, // Push Direct Page Register
    PLD, // Pull Direct Page Register
    PHB, // Push Data Bank Register

    PLB, // Pull Data Bank Register
    PHK, // Push Program Bank Register
    PEA, // Push Effective Absolute Address
    PEI, // Push Effective Indirect Address
    PER, // Push Effective PC Relative Address

    PHK, // Push Program Bank Register
    PEA, // Push Effective Absolute Address
    PEI, // Push Effective Indirect Address
    PER, // Push Effective PC Relative Address
    PEA, // Push Effective Absolute Address
    PEI, // Push Effective Indirect Address
    PER, // Push Effective PC Relative Address

    // Flags Manipulation
    CLC, // Clear Carry Flag
    SEC, // Set Carry Flag
    CLD, // Clear Decimal Mode Flag
    SED, // Set Decimal Mode Flag
    CLI, // Clear Interrupt Disable Flag
    SEI, // Set Interrupt Disable Flag
    CLV, // Clear Overflow Flag
    REP, // Reset Processor Status Bits
    SEP, // Set Processor Status Bits
    XCE, // Exchange Carry and Emulation bits

    // Block Move
    MVP, // Move Negative / Move Previous
    MVN, // Move Positive / Move Next

    // System Control, Interrupts and Other
    BRK, // Break
    COP, // Co-Processor Enable
    NOP, // No Operation
    WAI, // Wait for Interrupt
    STP, // Stop the Clock
    XBA, // Exchange B and A
    WDM, // Reserved
}
