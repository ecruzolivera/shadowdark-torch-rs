# Agents Configuration for shadowdark-torch-rs

## Project Overview
This file defines specialized AI agents for the shadowdark-torch-rs embedded Rust project - a firmware implementation that simulates Shadowdark RPG torch mechanics on ATtiny85 microcontrollers.

## Available Agents

### 🔧 Hardware & Embedded Development

#### embedded-expert
**Purpose**: Specialized in embedded Rust development for AVR microcontrollers
**Use for**:
- ATtiny85 hardware configuration and pin mapping
- AVR-specific optimizations and constraints
- Power management and sleep mode implementation
- Memory usage analysis and optimization
- Hardware abstraction layer (HAL) usage
- PWM configuration and timer setup
- ADC configuration for entropy sources

**Tools**: Has access to AVR toolchain, embedded debugging, memory analysis
**Expertise**: AVR architecture, embedded-hal traits, no_std development, hardware timers

#### hardware-tester
**Purpose**: Hardware validation and testing procedures
**Use for**:
- Creating test procedures for hardware functionality
- Debugging hardware issues and pin configurations
- Validating PWM output and LED behavior
- Testing power consumption and sleep modes
- Hardware-in-the-loop testing strategies
- Troubleshooting flashing and programming issues

**Tools**: Access to measurement tools, oscilloscope analysis, power profiling
**Expertise**: Hardware debugging, signal analysis, embedded testing methodologies

### 🦀 Rust Development

#### rust-optimizer
**Purpose**: Rust code optimization for embedded constraints
**Use for**:
- Size optimization (flash/RAM usage)
- Performance optimization for 1MHz CPU
- Unsafe code review and optimization
- Compile-time optimizations
- LTO and panic handler configuration
- Custom allocator implementation if needed

**Tools**: Rust compiler analysis, size profiling, performance benchmarking
**Expertise**: Embedded Rust, no_std optimization, AVR target specifics

#### rust-quality
**Purpose**: Code quality, safety, and best practices
**Use for**:
- Code review and refactoring suggestions
- Safety analysis for embedded code
- Documentation improvement
- Error handling in no_std environments
- API design for embedded systems
- Clippy and rustfmt configuration

**Tools**: Static analysis, linting, documentation generation
**Expertise**: Rust best practices, embedded safety, API design

### 🎲 Game Mechanics & Logic

#### shadowdark-mechanics
**Purpose**: Shadowdark RPG rule implementation and validation
**Use for**:
- Implementing accurate torch timing mechanics
- Calculating degradation curves and probability tables
- Adding new torch types or variants
- Rule validation against Shadowdark RPG specs
- Balancing game mechanics for hardware constraints
- Implementing additional RPG mechanics

**Tools**: Game rule databases, probability calculations, timing analysis
**Expertise**: Shadowdark RPG rules, game balance, tabletop RPG mechanics

#### torch-simulator
**Purpose**: Torch behavior simulation and modeling
**Use for**:
- Realistic flame flickering algorithms
- Time-based degradation modeling
- Random behavior tuning
- Visual effect optimization
- Brightness curve calculation
- Adding environmental effects (wind, rain, etc.)

**Tools**: Mathematical modeling, simulation frameworks, visual analysis
**Expertise**: Physical modeling, random number generation, visual effects

### 🔬 Testing & Validation

#### ci-cd-expert
**Purpose**: Continuous integration and deployment optimization
**Use for**:
- GitHub Actions workflow optimization
- Automated testing strategies
- Release automation
- Cross-compilation testing
- Artifact management and distribution
- Quality gate implementation

**Tools**: CI/CD platforms, automated testing, release management
**Expertise**: DevOps, automated testing, release engineering

#### firmware-validator
**Purpose**: Firmware validation and verification
**Use for**:
- Functional testing of torch behavior
- Timing accuracy validation
- Memory usage verification
- Power consumption testing
- End-to-end behavior validation
- Regression testing

**Tools**: Hardware testing rigs, automated validation, metrics collection
**Expertise**: Embedded testing, validation methodologies, quality assurance

### 📚 Documentation & User Experience

#### documentation-specialist
**Purpose**: Technical documentation and user guides
**Use for**:
- README and setup guide improvements
- Hardware assembly instructions
- Troubleshooting guides
- API documentation
- Circuit diagrams and schematics
- User manual creation

**Tools**: Documentation generators, diagram tools, technical writing
**Expertise**: Technical writing, embedded documentation, user experience

#### project-architect
**Purpose**: High-level project structure and architecture decisions
**Use for**:
- Project organization and structure
- Module design and separation of concerns
- Build system optimization
- Dependency management
- Feature planning and roadmap
- Architecture reviews

**Tools**: Project analysis, dependency management, architecture planning
**Expertise**: Software architecture, project management, technical leadership

## Usage Guidelines

### When to Use Specific Agents

1. **For hardware issues**: Start with `embedded-expert`, escalate to `hardware-tester` for validation
2. **For code optimization**: Use `rust-optimizer` for performance, `rust-quality` for maintainability  
3. **For game mechanics**: Use `shadowdark-mechanics` for rules, `torch-simulator` for behavior
4. **For testing**: Use `firmware-validator` for functional tests, `ci-cd-expert` for automation
5. **For documentation**: Use `documentation-specialist` for user-facing docs, `project-architect` for technical design

### Agent Collaboration

Agents can be used together for complex tasks:
- `embedded-expert` + `rust-optimizer` for hardware-optimized code
- `shadowdark-mechanics` + `torch-simulator` for accurate game implementation
- `firmware-validator` + `ci-cd-expert` for automated quality assurance
- `documentation-specialist` + `project-architect` for comprehensive project documentation

### Getting Started

To use an agent, specify the agent name when requesting assistance:
```
@embedded-expert Help optimize PWM frequency for realistic flickering
@shadowdark-mechanics Implement rules for different torch types
@rust-optimizer Reduce binary size while maintaining functionality
```

## Project-Specific Context

**Hardware Target**: ATtiny85 @ 1MHz
**Memory Constraints**: 8KB flash, 512B RAM
**Key Features**: PWM LED control, timer interrupts, sleep modes, PRNG
**Build Tools**: Cargo, Just, AVR toolchain
**Game System**: Shadowdark RPG torch mechanics