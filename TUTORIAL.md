# VIA RS 教程：用 Rust 构建可校验的电路模型

## 序言
硬件设计中最容易踩坑的环节，莫过于在电路拓扑尚未完全明确时，就仓促生成 KiCad、LCEDA 或 PCB 工程文件。文件看似能正常生成、在软件中顺利打开，但一旦执行封装关联、PCB 同步或导入 LCEDA Pro 等操作，各类问题就会集中爆发：引脚与焊盘无法对应、电源错接至逻辑管脚、模块多焊盘引脚在模型中缺失……

遇到这类问题，我们常常归咎于 EDA 工具难用，但本质原因是设计之初没有建立一套严谨、完整的电路模型。

`via-rs` 的核心目标从来不是“一键生成完美 PCB”——这件事在现阶段既不现实，也无必要。它真正解决的是一个更基础、也更关键的问题：**用 Rust 清晰定义电路的全部事实，在进入 EDA 工具之前，提前排查所有低级错误。**

所谓“电路事实”，至少包含以下维度：
- 电路板的名称与基本属性；
- 设计用到的全部器件；
- 每个器件的逻辑引脚定义；
- 逻辑引脚到物理焊盘的映射关系；
- 原理图中该器件应如何显示；
- PCB 中该封装的真实物理形状；
- 每个引脚的电气类型（电源、地、信号等）；
- 引脚所属的网络连接；
- 尚未经过实物测量确认的封装尺寸；
- 缺失厂家型号（MPN）或供应商料号（如 LCSC 编号）的器件；
- 以上事实能否无损导出为 KiCad、LCEDA Pro、VSCode 等工具兼容的格式。

本教程将围绕这些“事实”展开。它不是枯燥的命令清单，也不是追求快速演示的浅层入门。我们会遵循工科教材的思路：先理清核心概念，再给出代码示例，接着讨论常见问题，最后介绍具体命令与导出流程。

如果只想快速跑通命令，可以直接跳至附录；但如果希望将 `via-rs` 作为通用、可维护的生产力工具，建议从第 1 章开始阅读。这套系统的真正价值，不在于几个便捷的函数，而在于其背后严谨的建模规范。

## 目录
- [第 1 章 电路、器件和网络](#第-1-章-电路器件和网络)
- [第 2 章 第一个 VIA 电路](#第-2-章-第一个-via-电路)
- [第 3 章 逻辑引脚、原理图符号与 PCB 封装](#第-3-章-逻辑引脚原理图符号与-pcb-封装)
- [第 4 章 电气类型](#第-4-章-电气类型)
- [第 5 章 Typed Part：告别字符串引脚引用](#第-5-章-typed-part告别字符串引脚引用)
- [第 6 章 Footprint Pad Facts：封装焊盘事实](#第-6-章-footprint-pad-facts封装焊盘事实)
- [第 7 章 Pattern：将重复电路抽象为可复用结构](#第-7-章-pattern将重复电路抽象为可复用结构)
- [第 8 章 检查与诊断](#第-8-章-检查与诊断)
- [第 9 章 从零新增一个示例工程](#第-9-章-从零新增一个示例工程)
- [第 10 章 导出 KiCad、LCEDA Pro 与快照文件](#第-10-章-导出-kicadlceda-pro-与快照文件)
- [第 11 章 生产就绪度](#第-11-章-生产就绪度)
- [第 12 章 常见错误与排查路径](#第-12-章-常见错误与排查路径)
- [第 13 章 完整工作流与 crate 分层](#第-13-章-完整工作流与-crate-分层)
- [第 14 章 Component 生命周期与 typed API](#第-14-章-component-生命周期与-typed-api)
- [第 15 章 规则、线宽、间距与过孔](#第-15-章-规则线宽间距与过孔)
- [第 16 章 电源 Rail 与去耦电容](#第-16-章-电源-rail-与去耦电容)
- [第 17 章 封装生成器实战](#第-17-章-封装生成器实战)
- [第 18 章 测试、CI 与版本演进](#第-18-章-测试ci-与版本演进)
- [附录 A 当前 API 速查](#附录-a-当前-api-速查)
- [附录 B 常用命令](#附录-b-常用命令)

## 第 1 章 电路、器件和网络
在展开高级功能之前，必须先明确系统的核心对象与定义，否则后续所有操作都会失去可靠的根基。在正式设计复杂电路之前，我们先统一概念：在 `via-rs` 的体系中，电路到底由哪些部分组成？

本章将理清以下核心概念：
- `Design` 与 `Board`；
- 器件（part / module）；
- 逻辑引脚；
- 物理焊盘；
- 网络；
- 封装事实；
- 区分以上概念的必要性。

### 1.1 `Design` 是什么
`Design` 是编写代码、搭建电路的核心载体，相当于一份处于迭代中的工程草案：你可以向其中添加器件、定义网络连接、配置设计规则，在最终定稿前均可自由修改。

一个最小的 `Design` 定义如下：
```rust
use via::prelude::*;

let mut d = Design::new("demo_board")
    .rules(Rules::new())
    .units(Unit::Mm);
```
其中 `demo_board` 是电路板名称，`Rules::new()` 加载基础设计规则（线宽、过孔、间距等），`Unit::Mm` 指定几何单位为毫米。硬件领域常提及 mil 单位，但本项目中所有几何尺寸统一使用毫米。

需要注意的是，`Design` 并非电路的最终形态。当器件、网络、封装全部定义完成后，需要调用：
```rust
let board = d.build()?;
```
生成最终的 `Board` 模型。

### 1.2 `Board` 是什么
`Board` 是经过规则校验后生成的**只读**电路模型。导出 KiCad 文件、LCEDA Pro 文件、JSON 快照或进行 PCB 布局时，所有导出器仅读取 `Board` 中的确定信息，无需再推断 `Design` 中的设计意图。

如果用编程概念类比：`Design` 是仍在编写与调试的函数定义，`Board` 则是通过编译与校验后的可执行函数；而 KiCad 文件、LCEDA 文件与快照 JSON，只是同一函数在不同格式下的导出形态。

系统遵循一条核心原则：
> **所有 KiCad/LCEDA 导出器、VSCode 插件均不应自行补充电路逻辑，必须读取同一份 `Board`（或其导出的快照）作为唯一数据源。**

违背这一原则，就会回到难以维护的混乱状态：原理图一套引脚映射、PCB 一套、导出脚本再偷偷补一套，三处定义只要有一处不一致，问题就会在导入、同步或布线阶段集中爆发。

### 1.3 器件是什么
器件是电路板上的实体功能单元，小至一颗贴片电阻，大到 ESP32 开发板、TMC2209 驱动模块、DC005 电源插座，都属于器件范畴。

在 `via-rs` 中，一个完整的器件定义至少需要包含以下信息：
- 位号（reference designator）：如 `R1`、`U2`、`J1`；
- 参数/描述（value）：如 `1k`、`100nF`、`TMC2209 v2.0`；
- 封装名（footprint）；
- 全部逻辑引脚定义；
- 逻辑引脚到物理焊盘的映射；
- 引脚的电气类型；
- 生产备注：例如封装是否需要实物测量验证。

以一个简单的电阻为例：
```rust
part("R1", "1k")
    .footprint("R_0805_2012Metric")
    .pin(pin("1").passive().pad("1"))
    .pin(pin("2").passive().pad("2"))
```
位号 `R1`，阻值 `1k`，封装为 0805。两个逻辑引脚 `1` 和 `2`，分别对应物理焊盘 `1` 和 `2`。

这类简单器件的逻辑引脚与物理焊盘命名完全一致，看起来似乎有些冗余。但请不要被简单场景迷惑：遇到 ESP32、TMC2209 等复杂模块时，逻辑引脚与物理焊盘的命名往往差异极大。如果不从一开始就养成区分二者的习惯，后续排查连线错误会非常困难。

### 1.4 逻辑引脚是什么
逻辑引脚面向设计人员，对应引脚在电路中的实际功能。

以 TMC2209 为例，设计中真正关注的是这类命名：
```text
VMOT, GND, VIO, STEP, DIR, UART, OA1, OA2, OB1, OB2
```
通过这些命名可以直接识别引脚用途：`VMOT` 接电机电源、`STEP` 输入步进脉冲、`OA1/OA2` 连接电机相线。

如果代码中只能写：
```rust
u2.pin("9")
u2.pin("15")
```
代码可读性会急剧下降。`9` 号引脚对应什么信号？时隔两周再回看，很难快速还原当时的设计意图。因此我们更推荐这样的写法：
```rust
driver.vmot()
driver.vio()
```
这就是类型化器件句柄（typed part handle）的价值，第 5 章会详细介绍。

### 1.5 物理焊盘是什么
物理焊盘面向 EDA 工具与 PCB 生产，对应封装库中的焊盘编号与实际坐标。

仍以 TMC2209 为例，逻辑上的 `VMOT` 实际对应封装的 9 号引脚，`VIO` 对应 15 号引脚。因此模型必须明确定义映射关系：
```rust
.pin(pin("VMOT").power("12V").pad("9"))
.pin(pin("VIO").power("3V3").pad("15"))
```

单个逻辑引脚也可对应多个物理焊盘，例如模块的接地引脚往往存在多个焊盘：
```rust
.pin(pin("GND").ground().pads(["10", "16"]))
```
这句话的含义是：原理图逻辑上它是一个统一的地网络，但物理封装上对应 10 号和 16 号两个焊盘，PCB 布线时二者均可接地。

这一映射关系是电路模型的核心细节。LCEDA 导入时常见的“引脚与焊盘未对应”报错，大多源于此处的定义偏差——并非 EDA 工具校验严苛，而是模型未明确逻辑与物理的对应关系。

### 1.6 网络是什么
网络是引脚之间的电气连接关系。例如：
```rust
let v12 = d.power("12V_IN", Voltage::dc(12.0));
let gnd = d.ground("GND");
let x_step = d.logic("X_STEP", "3V3");
```
这三行代码分别创建了三个网络：12V 电源网络、接地网络、3.3V 逻辑信号网络。

之后将引脚接入对应网络：
```rust
v12.connect_all(&mut d, [jack.tip_12v(), driver.vmot()]);
gnd.connect_all(&mut d, [jack.sleeve_ground_verify(), driver.ground()]);
x_step.connect_all(&mut d, [esp32.gpio7(), driver.step()]);
```

需要注意的是，`v12`、`gnd` 等变量并非网络的可变引用，而是轻量级的网络句柄（NetHandle）。这种设计避开了 Rust 借用检查器的限制，你可以先定义所有网络句柄，再逐步完成连线，代码结构更清晰。

### 1.7 封装事实是什么
假设你定义了如下器件：
```rust
part("U4", "MP1584 buck adapter")
    .footprint("BuckModule_4Wire_MP1584_Adapter")
    .pin(pin("IN+").power("12V").pad("1"))
```
这段代码指定 `IN+` 映射到 1 号焊盘，但系统无法默认该封装确实存在 1 号焊盘——实际封装的焊盘命名可能是 `A`、`B`，也可能与模型定义不一致。

这就需要“封装焊盘事实（Footprint Pad Facts）”作为校验基准。最简的封装事实定义如下：
```rust
d.add_footprint_pads(FootprintPads::new(
    "BuckModule_4Wire_MP1584_Adapter",
    ["1", "2", "3", "4"],
));
```
这段代码不涉及焊盘的形状、坐标等几何信息，仅向检查器声明一个事实：该封装确实存在 `1, 2, 3, 4` 四个焊盘。

当前项目中，绝大多数封装事实由代码自动生成：
```rust
for footprint in parts::generated_footprint_pads() {
    d.add_footprint_pads(footprint);
}
```

基于这些事实，系统可以在导出文件前完成前置校验，提前排除三类典型问题：
- 逻辑引脚映射的物理焊盘是否真实存在；
- 封装上是否存在未被任何逻辑引脚覆盖的“孤儿”焊盘；
- 是否出现多个网络连接到同一个物理焊盘的短路问题。

这正是 `via-rs` 的核心价值：**让低级错误在终端编译阶段就被拦截，而不是留到 EDA 工具中才暴露。**

### 1.8 本章小结
虽然尚未开始实际的 PCB 设计，但请牢记以下核心原则：
1. `Design` 是电路搭建的草稿载体，`Board` 是校验完成后的确定事实集合。
2. 器件必须声明完整的逻辑引脚。
3. 逻辑引脚必须准确映射到物理焊盘。
4. 网络连接的对象是逻辑引脚，最终 PCB 铺铜作用于物理焊盘。
5. 封装事实是检查器校验引脚匹配问题的基础。
6. 导出器仅负责格式转换，绝对不应二次定义电路逻辑。

## 第 2 章 第一个 VIA 电路
明确核心概念后，我们通过一个极简电路上手实操。这个电路不具备实际功能，仅用于熟悉基本语法：
- 一个 3 针排针作为输入；
- 一个 3 针负载模块；
- 一路 3.3V 信号网络；
- 一路 3.3V 电源网络；
- 一路接地网络。

### 2.1 完整代码
```rust
use via::prelude::*;

pub fn board() -> Result<Board> {
    let mut d = Design::new("demo_board")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = d.logic("SIGNAL", "3V3");
    let v3v3 = d.power("3V3", Voltage::dc(3.3));
    let gnd = d.ground("GND");

    let header = d.add(
        part("J1", "External signal input")
            .footprint("Header_1x03")
            .pin(pin("SIG").logic("3V3"))
            .pin(pin("3V3").power("3V3"))
            .pin(pin("GND").ground()),
    )?;

    let load = d.add(
        part("U1", "Demo load")
            .footprint("Demo_Load_3Pin")
            .pin(pin("IN").logic("3V3"))
            .pin(pin("VCC").power("3V3"))
            .pin(pin("GND").ground()),
    )?;

    signal.connect_all(&mut d, [header.pin("SIG"), load.pin("IN")]);
    v3v3.connect_all(&mut d, [header.pin("3V3"), load.pin("VCC")]);
    gnd.connect_all(&mut d, [header.pin("GND"), load.pin("GND")]);

    d.check(CheckProfile::Prototype)?;
    d.build()
}
```

### 2.2 逐行拆解
首行引入预定义模块，导入常用的类型与方法：
```rust
use via::prelude::*;
```

创建设计草案：
```rust
let mut d = Design::new("demo_board")
    .rules(Rules::new())
    .units(Unit::Mm);
```

定义三类网络：
```rust
let signal = d.logic("SIGNAL", "3V3");
let v3v3 = d.power("3V3", Voltage::dc(3.3));
let gnd = d.ground("GND");
```
注意三类网络的区别：`SIGNAL` 是 3.3V 电平的逻辑信号网络，`3V3` 是电源网络，`GND` 是参考地，三者电气属性完全不同。分类标注的目的，是让检查器能自动拦截电源与信号短路等错误连接。

添加器件 `J1`：
```rust
let header = d.add(
    part("J1", "External signal input")
        .footprint("Header_1x03")
        .pin(pin("SIG").logic("3V3"))
        .pin(pin("3V3").power("3V3"))
        .pin(pin("GND").ground()),
)?;
```
示例中省略了 `.pad("1")` 的显式声明。在最简测试场景下，若未指定物理焊盘，系统会默认将逻辑引脚名直接作为焊盘名。但在对接真实 EDA 工具或正式设计中，必须显式声明实际焊盘编号：
```rust
    .pin(pin("SIG").logic("3V3").pad("1"))
```

完成所有连线后，执行原型检查并生成最终 Board：
```rust
    // ... connect_all 连线 ...
    d.check(CheckProfile::Prototype)?;
    d.build()
```

### 2.3 简单校验测试
尝试将连线代码：
```rust
signal.connect_all(&mut d, [header.pin("SIG"), load.pin("IN")]);
```
故意修改为：
```rust
signal.connect_all(&mut d, [header.pin("SIG"), load.pin("INPUT")]);
```
运行后检查器会立即抛出 `unknown pin` 错误。这并非导出器或 KiCad 的问题，而是电路模型在构建阶段就识别出了不存在的引脚引用。

## 第 3 章 逻辑引脚、原理图符号与 PCB 封装
逻辑引脚与物理焊盘的映射是 `via-rs` 最核心也最易被忽略的设计。可以这样理解：逻辑引脚是面向人的“功能地名”，物理焊盘是面向机器的“坐标编号”，二者必须建立严格的对应关系。

### 3.1 不要被简单器件迷惑
电阻这类简单器件的逻辑引脚与焊盘命名一致，很容易让人忽略二者的区别：
```rust
.pin(pin("1").passive().pad("1"))
```
但遇到 MP1584 电源模块时，设计人员只认 `IN+ / IN-` 的功能命名，而板厂封装仅识别 `1 / 2` 的编号，必须建立明确映射：
```rust
part("U4", "MP1584 buck adapter")
    .footprint("BuckModule_4Wire_MP1584_Adapter")
    .pin(pin("IN+").power("12V").pad("1"))
    .pin(pin("IN-").ground().pad("2"))
    // ...
```
这并非冗余定义，而是为了避免 EDA 工具、导出脚本与设计人员三者之间出现信息偏差。

### 3.2 一对多：多焊盘逻辑引脚
对于模块、连接器类器件，单个逻辑引脚通常对应多个物理焊盘。例如 ESP32-S3 开发板的接地引脚就包含 4 个物理焊盘：
```rust
.pin(pin("GND").ground().pads(["22", "23", "43", "44"]))
```
这表示原理图中它们统一属于同一个地网络，但 PCB 设计时 4 个焊盘均可接地布线。

### 3.3 反面教材：零散定义方式
早期的零散定义方式如下：
```rust
Part::new("U4", "MP1584")
    .pins(["IN+", "IN-"])
    .pinmap([("IN+", "1"), ("IN-", "2")])
    .power_pin("IN+", "12V")
```
这种写法并非不可行，但属性分散在多处，修改引脚名时很容易遗漏更新，埋下不一致的隐患。更推荐的写法是将同一引脚的所有属性聚合定义：
```rust
.pin(pin("IN+").power("12V").pad("1"))
```

### 3.4 LCEDA 引脚报错的排查思路
如果导入 LCEDA Pro 时弹出报错：
```text
元件 U4 的引脚与焊盘未对应
引脚没有对应焊盘：IN+、IN-
焊盘没有对应引脚：1、2
```
先不要急于归因于格式问题，请按以下步骤自查：
1. 找到 `U4` 的器件定义函数；
2. 确认逻辑引脚是否声明了对应的物理焊盘；
3. 确认封装事实中是否包含这些焊盘编号；
4. 运行 Rust 侧的检查命令，确认模型本身是否自洽；
5. 若 Rust 模型无问题，再排查 LCEDA 导出器逻辑。

排查原则：先校验模型定义，再排查工具与导出逻辑。

### 3.5 `part` 不是原理图符号，也不是 PCB 封装
这里必须把一个非常重要的概念讲清楚：`part(...)` 定义的是**电路语义**，不是原理图上的图形，也不是 PCB 上的铜皮形状。

同一个器件在系统中至少有四层表示：

| 层级 | 负责什么 | 不负责什么 |
| --- | --- | --- |
| `Part` / module | 位号、值、逻辑引脚、电气类型、逻辑引脚到焊盘的映射、生产备注 | 不决定原理图图形怎么画，也不决定焊盘的坐标和形状 |
| Schematic symbol | 原理图上怎么显示：矩形、引脚左右分布、引脚名、引脚号、位号和值的位置 | 不定义铜皮、孔、焊盘尺寸，也不应该重新定义电气连接 |
| Footprint Pad Facts | 这个 footprint 有哪些焊盘编号，用于检查 pin-pad 映射 | 不定义焊盘坐标、形状、孔径、丝印 |
| PCB footprint geometry / `FootprintIr` | PCB 上的真实物理几何：焊盘、孔、丝印、装配层、文本、外框 | 不决定这个器件在电路中应该接到哪个网络 |

所以这段代码：

```rust
part("U4", "MP1584 buck adapter")
    .footprint("BuckModule_4Wire_MP1584_Adapter")
    .pin(pin("IN+").power("12V").pad("1"))
```

表达的是：

1. 电路中有一个位号为 `U4` 的模块；
2. 它的描述是 `MP1584 buck adapter`；
3. 它引用名为 `BuckModule_4Wire_MP1584_Adapter` 的 PCB 封装；
4. 它有一个逻辑引脚 `IN+`；
5. `IN+` 是 12V 电源输入；
6. `IN+` 对应 PCB 封装中的 `1` 号物理焊盘。

它没有表达：

1. 原理图上 `IN+` 应该画在左边还是右边；
2. 原理图符号外框多高多宽；
3. PCB 上 `1` 号焊盘在什么坐标；
4. `1` 号焊盘是圆形、矩形、长圆孔还是贴片焊盘；
5. 丝印、装配层、`REF**`、value 文本放在哪里。

这些内容必须分别由 symbol style 和 footprint geometry 处理。把这些层混在一起，是很多 EDA 导入问题的根源。

### 3.6 原理图上的样式如何设计
原理图符号是“给人读电路用的视觉表示”。它的目标不是还原器件外形，而是让电路逻辑清楚。

当前 `via-rs` 的 KiCad/LCEDA 导出器采用的是自动符号生成策略：根据 `Part` 的逻辑引脚生成一个矩形符号，将引脚排列在左右两侧，显示逻辑引脚名，并把物理焊盘号作为 pin number。也就是说，当前原理图符号主要由以下信息推导：

```text
Part
  -> logical pins
  -> pad mapping
  -> generated schematic symbol
```

这解释了一个现象：你在原理图上看到的是 `IN+`、`VMOT`、`GPIO7` 这类功能名称，而不是单纯的一串 `1`、`2`、`3`。这是正确的。原理图是给人看功能的，不是给 PCB 机床看坐标的。

更成熟的 symbol style 后续应当支持：

- 将电源引脚放在上方或下方；
- 将输入引脚放左侧、输出引脚放右侧；
- 将电机相线放在同一侧；
- 隐藏 NC 引脚或将其集中分组；
- 单元拆分，例如一个复杂芯片拆成 power unit、logic unit、driver unit；
- 位号、值、footprint 属性的位置和可见性；
- 引脚名与引脚号是否显示。

但要注意，原理图符号样式不应该成为新的电气事实源。符号可以决定“这个 pin 画在哪里”，但不能决定“这个 pin 接什么网络”。电气连接仍然来自 `Part` 和 `Net`。

简单说：

```text
原理图 symbol 解决的是：人如何读懂电路。
```

它不是 PCB 制造几何，也不是 pin map 的第二份副本。

### 3.7 PCB 上的物理样式如何设计
PCB 封装是“给布线、DRC 和制造用的物理表示”。它必须描述真实世界中的尺寸。

当前 `via-rs` 用 `FootprintIr` 表达 PCB 封装几何。一个 footprint 至少包含：

- `Pad`：焊盘编号、焊盘类型、形状、坐标、尺寸、孔径、所在层；
- `GraphicLine`：丝印层、装配层、外框线等；
- `GraphicText`：reference、value、用户文字；
- metadata：生成器名称、备注、是否需要验证等。

例如一个简化的 0805 两端器件 footprint 可以这样理解：

```rust
let mut fp = FootprintIr::new("R_0805_2012Metric");

fp.add_pad(Pad::smd(
    "1",
    PadShape::Rect,
    Point::new(-0.95, 0.0),
    Size::new(1.15, 1.35),
));

fp.add_pad(Pad::smd(
    "2",
    PadShape::Rect,
    Point::new(0.95, 0.0),
    Size::new(1.15, 1.35),
));

fp.add_rect(
    Point::new(-1.0, -0.65),
    Point::new(1.0, 0.65),
    "F.Fab",
    0.08,
);

fp.add_text(GraphicText::reference("REF**", Point::new(0.0, -1.2), "F.SilkS"));
fp.add_text(GraphicText::value("R_0805_2012Metric", Point::new(0.0, 1.2), "F.Fab"));
```

这段代码描述的是 PCB 上的真实几何：

1. `1` 号焊盘在 `x = -0.95 mm`；
2. `2` 号焊盘在 `x = +0.95 mm`；
3. 两个焊盘都是 SMD 矩形焊盘；
4. 每个焊盘尺寸为 `1.15 x 1.35 mm`；
5. 外框画在 `F.Fab`；
6. 位号使用 `Reference` text，而不是普通 user text；
7. value 使用 `Value` text，而不是把 footprint 名字随便画成高亮文字。

再看 DC005 这类连接器。它不是简单圆孔排针，通常包含机械固定脚、长圆孔、开关脚、外壳定位等结构。因此不能随便用几个圆形通孔糊上去。它应该用类似下面的物理元素表达：

```rust
Pad::thru_hole_slot(
    "4",
    PadShape::Oval,
    Point::new(7.5, 0.0),
    Size::new(3.0, 1.8),
    2.2,
    0.9,
)
```

这里 `thru_hole_slot` 表示长圆孔，`PadShape::Oval` 表示长圆焊盘，`Point` 表示焊盘中心坐标，`Size` 表示焊盘铜皮尺寸，最后两个参数表示槽孔尺寸。这样的定义才接近真实器件，而不是把所有孔都画成圆形。

PCB footprint 的设计步骤通常是：

1. 找数据手册或实物图纸；
2. 确认所有 pad number；
3. 确认 pad kind：SMD、通孔、非金属孔；
4. 确认 pad shape：圆形、矩形、圆角矩形、长圆、梯形；
5. 确认 pad 坐标、尺寸和孔径；
6. 画 `F.Fab` 外形，表示器件本体尺寸；
7. 画 `F.SilkS` 丝印，表示 PCB 上可见轮廓；
8. 放置 `Reference` 和 `Value` 文本；
9. 从 `FootprintIr` 自动抽取 `FootprintPads`；
10. 用检查器验证 part 的 pin-pad 映射。

简单说：

```text
PCB footprint 解决的是：板子如何制造和布线。
```

### 3.8 正确的设计流程
因此，一个器件在 `via-rs` 中最理想的设计流程是：

```text
1. 先定义 Part
   - refdes
   - value
   - logical pins
   - electrical classes
   - logical pin -> physical pad mapping

2. 再定义 schematic symbol style
   - pin 分组
   - 左右上下布局
   - pin name / pin number 显示策略
   - reference / value 属性位置

3. 再定义 PCB footprint geometry
   - pads
   - drills
   - layers
   - silk/fab/courtyard
   - reference/value/user text

4. 从 footprint geometry 抽取 Footprint Pad Facts
   - 检查 part 写到的 pad 是否真实存在
   - 检查 footprint 是否有未覆盖 pad

5. 最后导出
   - KiCad schematic
   - LCEDA Pro project
   - VSCode snapshot
   - KiCad PCB draft
```

这条流程的关键在于：每层只负责自己的事情。

`Part` 不关心焊盘画多大；`Symbol` 不关心孔径；`FootprintIr` 不关心这个 pad 最终接 `12V_IN` 还是 `GND`；导出器只负责把这些事实翻译成目标 EDA 的格式。

这样设计出来的库才容易维护，也更容易扩展。后续如果要做更漂亮的原理图样式，就扩展 symbol style；如果要修 DC005 的长圆孔，就改 footprint generator；如果 LCEDA 报 pin-pad 错，就查 `Part` 与 `FootprintPads` 的对应关系。每类问题都有明确归属。

## 第 4 章 电气类型
电路设计绝非简单的引脚连线。如果不对电源、地、逻辑信号做电气属性区分，很容易出现 12V 电源直连 GPIO 等致命错误，造成器件烧毁。

### 4.1 常用电气类型标注
常用的电气类型声明如下：
```rust
pin("GND").ground()        // 接地
pin("VIN").power("12V")    // 电源，指定电压域
pin("STEP").logic("3V3")   // 逻辑信号，指定电平
pin("1").passive()         // 被动器件引脚
pin("OA1").motor_phase()   // 电机相线
```
如果将标记为 `logic:3V3` 的引脚强行连接到 `power:12V` 的网络，检查器会直接拦截该错误连接。

### 4.2 被动器件为何使用 passive
电阻、电容等被动器件不要随意标注电气类型。以 10k 电阻为例，它既可作为上拉电阻接 3V3，也可串联在 GPIO 信号线上；若错误标记为 `logic("3V3")`，在其他合法连接场景下就会产生误报。统一使用 `passive()` 即可。

### 4.3 未确定的引脚不要强行标注
对于 TMC2209 的 `MS1/MS2` 这类配置引脚，最终可能接地也可能接高电平，若尚未确定连接方式，可仅声明引脚存在：
```rust
pin("MS1").pad("7")
```
这并非省略定义，错误的类型标注比不标注带来的问题更严重。

## 第 5 章 Typed Part：告别字符串引脚引用
如果每次引用引脚都需要手写字符串：
```rust
esp.pin("GPIO4")
```
不仅开发效率低，还极易因拼写错误引发问题，和手工编写网表没有本质区别。更理想的方式是通过方法直接引用引脚：
```rust
esp.gpio4()
```

### 5.1 封装带类型句柄的器件
以一个 I2C 传感器为例：
```rust
use via::prelude::*;

#[derive(Debug, Clone)]
pub struct Sensor {
    id: ModuleId,
}

impl Sensor {
    pub fn vcc(&self) -> PinRef { self.id.pin("VCC") }
    pub fn gnd(&self) -> PinRef { self.id.pin("GND") }
    pub fn scl(&self) -> PinRef { self.id.pin("SCL") }
    pub fn sda(&self) -> PinRef { self.id.pin("SDA") }
}

pub fn sensor(refdes: &str) -> impl Component<Output = Sensor> {
    part(refdes, "I2C sensor module")
        .footprint("Sensor_1x04")
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .verify()
        .handle(|id| Sensor { id }) // 核心：生成类型化句柄
}
```

通过 `.handle(|id| Sensor { id })` 方法，普通器件被封装为带类型的句柄实体，连线时可以直接调用对应方法，代码意图更清晰，同时能获得编辑器的自动补全支持：
```rust
let sensor = d.add(sensor("U5"))?;

v3v3.connect(&mut d, sensor.vcc());
gnd.connect(&mut d, sensor.gnd());
i2c_scl.connect(&mut d, sensor.scl());
```

### 5.2 封装的边界
不必追求完整封装器件的所有引脚——例如无需将 ESP32 的数百个引脚全部手动实现。仅针对当前设计用到的引脚提供方法即可，API 的核心作用是清晰表达设计意图，而非完整复刻数据手册。

## 第 6 章 Footprint Pad Facts：封装焊盘事实
第 3 章介绍了引脚映射的基本规则，而要让检查器验证映射的正确性，还需要引入封装焊盘事实（Footprint Pad Facts）作为基准。

### 6.1 什么是封装焊盘事实
封装焊盘事实并非完整的封装几何定义，而是最基础的元信息集合：仅声明该封装包含哪些焊盘编号。

例如：
```rust
FootprintPads::new(
    "BuckModule_4Wire_MP1584_Adapter",
    ["1", "2", "3", "4"],
)
```
这段定义不涉及焊盘的位置、形状、尺寸等几何信息，仅说明该封装存在 `1`、`2`、`3`、`4` 四个物理焊盘。

可以将其理解为封装的“最小校验摘要”，基于这份摘要，`via-rs` 可以校验三类问题：
- 逻辑引脚映射的物理焊盘是否存在；
- 封装上是否存在未被逻辑引脚覆盖的焊盘；
- 同一个物理焊盘是否被多个不同网络占用。

这三类问题看似基础，恰恰是导入 KiCad/LCEDA 时最常见的崩溃原因。

### 6.2 手动添加封装事实
对于临时使用的外部封装，可以手动声明焊盘事实：
```rust
d.add_footprint_pads(FootprintPads::new(
    "Header_1x03",
    ["1", "2", "3"],
));
```
这种方式适合设计早期阶段：在完成完整封装生成器之前，只要明确焊盘编号，就可以先补充焊盘事实以支持基础校验。

需要注意的是，手动声明仅能解决“焊盘是否存在”的校验问题，不包含任何几何信息，无法支持 VSCode 编辑器的焊盘可视化，也不能生成 KiCad 可用的封装文件。因此手动焊盘事实可作为过渡方案，不应作为复杂器件的长期维护方式。

### 6.3 从自动生成封装中获取事实
项目中绝大多数封装事实由自动生成的封装提供：
```rust
for footprint in parts::generated_footprint_pads() {
    d.add_footprint_pads(footprint);
}
```
该函数来自 harmonic 器件库，对应文件路径：
```text
C:\Coding\谐波赤道仪\tools\electronics\via-rs\crates\via-parts-harmonic\src\footprints.rs
```

其中包含两个核心函数：
```rust
pub fn generated_footprints() -> Vec<GeneratedFootprint>;
pub fn generated_footprint_pads() -> Vec<FootprintPads>;
```

三者的层级关系如下：
```text
GeneratedFootprint
    -> FootprintIr
        -> FootprintPads
```
`GeneratedFootprint` 可生成 `.kicad_mod` 封装文件，`FootprintIr` 保存完整几何结构，`FootprintPads` 则提取焊盘编号供核心检查器使用。

也就是说，同一套封装生成器同时支撑三个场景：
- 生成 KiCad 封装文件；
- 为 VSCode 快照提供几何数据；
- 为核心检查器提供焊盘事实。

这正是“单一事实源”设计理念的体现：避免 KiCad 导出器、VSCode 编辑器、LCEDA 导出器各自维护一份焊盘列表，从根源上消除信息不一致的风险。

### 6.4 可排查的典型错误
若写错焊盘编号：
```rust
.pin(pin("IN+").power("12V").pad("5")) // 实际封装仅含 1~4 号焊盘
```
检查器对照封装事实后，会立即抛出 `pin_pad_map.missing_pad` 错误。

反之，若封装包含 16 个焊盘，但器件定义中仅声明了 15 个逻辑引脚（例如遗漏了散热焊盘或空引脚），检查器会报 `pin_pad_map.uncovered_footprint_pad`，强制保证器件模型与实际封装完全对齐。

### 6.5 焊盘事实与完整封装的适用场景
如果仅需原理图级别的校验，焊盘事实已完全足够，可以验证逻辑引脚与物理焊盘的对应关系。

但如果需要实现以下功能，则需要完整的封装几何定义：
- 在 VSCode 编辑器中真实显示焊盘位置；
- 支持点击焊盘进行布线；
- 校验走线是否与焊盘正确连接；
- 导出 `.kicad_pcb` 文件；
- 生成可审查的 `.kicad_mod` 封装；
- 校验 DC005 等特殊形状的焊盘。

简言之，焊盘事实是校验的底线，完整封装几何是更进一步的物理描述。

### 本章小结
1. `FootprintPads` 仅描述焊盘的存在性，不包含几何信息。
2. `FootprintIr` 描述焊盘、孔、图形线、文字等 footprint 内部几何结构。
3. 检查器依赖 `FootprintPads` 排查引脚-焊盘映射错误。
4. 编辑器与 PCB 导出器依赖几何信息实现显示与导出。
5. 正式器件应尽量基于自动生成或实物测量的封装进行支撑。

## 第 7 章 Pattern：将重复电路抽象为可复用结构
单路 TMC2209 外围电路手写尚可接受，若要实现四路甚至更多，重复代码会显著增加，出错概率也随之上升。对于高复用的电路模块，可以将其抽象为 Pattern（可复用电路模板）。

### 7.1 Pattern 的价值
以两路 TMC2209 驱动电路为例，每一路都包含驱动模块、电机连接器、UART 串联电阻，以及控制信号、电机相线、电源地等连接。全部手写不仅代码冗长，还容易出现轴间差异：例如 X 轴添加了 UART 串联电阻而 Y 轴遗漏，或者某一轴的电机相线顺序写反，这类细微错误很难通过肉眼快速发现。

Pattern 的作用就是将这类重复结构封装起来，保证一致性的同时减少重复劳动。

### 7.2 TMC2209 UART 驱动轴示例
当前 Pattern 的典型写法如下：
```rust
let esp32 = d.add(parts::esp32_s3_n16r8("U1"))?;

let x_axis = d.add(
    patterns::Tmc2209UartAxisSpec::new("X")
        .driver("U2")
        .motor_connector("J2")
        .uart_resistor("R1")
        .pins(patterns::Tmc2209UartAxisPins::new(
            esp32.gpio4(),
            esp32.gpio5(),
            esp32.gpio6(),
            esp32.gpio7(),
            esp32.gpio15(),
        )),
)?;
```

这段代码创建了名为 `X` 的驱动轴，会自动生成对应器件、网络与连接，包括：
- `U2` TMC2209 驱动芯片；
- `J2` 电机连接器；
- `R1` UART 发送端串联电阻；
- `X_EN`、`X_UART_TX`、`X_UART`、`X_STEP`、`X_DIR` 等逻辑网络；
- `X_OA1`、`X_OA2`、`X_OB1`、`X_OB2` 等电机相线网络。

返回的 `x_axis` 并非黑盒，仍然暴露关键引脚方法：
```rust
x_axis.vmot()
x_axis.vio()
x_axis.ground()
x_axis.ms1()
x_axis.ms2()
```
用户仍可显式连接电源、地与配置引脚。

### 7.3 电源输入 Pattern
电源输入电路同样适合抽象为 Pattern。当前的 `DcBuckInputStageSpec` 模板包含直流插座、降压模块、输入输出滤波电容等完整结构，使用方式如下：
```rust
d.add(
    patterns::DcBuckInputStageSpec::new()
        .input_loads([x_axis.vmot(), y_axis.vmot()])
        .output_loads([esp32.power_5v()]),
)?;
```

Pattern 不会自动推断电源连接，用户仍需显式指定输入侧与输出侧的负载，保证电气连接意图清晰可控。

### 7.4 好 Pattern 与坏 Pattern
优秀的 Pattern（例如 `Tmc2209UartAxisSpec`）会封装重复的器件与连线，同时保留清晰的外部接口，让用户显式连接电源、地等关键网络：
```rust
v12.connect(&mut d, x_axis.vmot());
gnd.connect(&mut d, x_axis.ground());
```

糟糕的 Pattern 则会过度封装，甚至在内部隐式完成电源与地的连接。看似简化了操作，实则变成了黑盒，后续排查问题时很难追溯电源路径与连接关系。

判断一个 Pattern 是否合理，可以看三点：
1. 是否有效减少了重复代码；
2. 是否保留了关键的电气连接意图；
3. 是否返回了足够清晰的类型化接口。

如果一个 Pattern 让用户无法感知电源、地、关键连接和器件位号，就属于过度封装。

### 本章小结
- Pattern 适合封装高重复度的电路模块；
- Pattern 不应隐藏关键电气事实；
- Pattern 应返回类型化的输出句柄；
- Pattern 应保留用户显式连接核心网络的能力。

## 第 8 章 检查与诊断
`via-rs` 的各类检查机制并非为了增加使用门槛，而是为了将问题拦截在设计早期。错误发现得越早，修复成本越低；若等到导入 EDA 工具后再调整网表，定位与修改的成本会大幅提升。

### 8.1 原型检查
运行原型检查命令：
```powershell
cargo run -p via-cli -- check --example polar-adjuster --json
```

检查通过时会返回如下格式的结果：
```json
{
  "board": "polar_adjuster_v0",
  "ok": true,
  "footprints_loaded": 14,
  "diagnostics": []
}
```
这仅代表电路模型在原型层面结构自洽，不代表设计已达到投产标准。

### 8.2 生产检查
运行生产级门禁检查：
```powershell
cargo run -p via-cli -- check-production --example polar-adjuster --json
```
若设计中存在未经过实物测量验证的封装（带有 `VERIFY` 标记），或缺少完整的厂商型号、供应商料号，生产检查会直接报错拦截。

以当前的 `polar-adjuster` 为例，仍存在多个待验证封装，且缺少完整的 MPN 与供应商料号，生产检查报错属于正常情况。

### 8.3 诊断信息结构
每条诊断都不是孤立的文本，而是结构化的对象：
```json
{
  "severity": "error",
  "code": "production.unverified_footprint",
  "message": "J1 footprint DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY still requires physical verification before production",
  "object": {
    "kind": "module",
    "refdes": "J1"
  },
  "related": [
    {
      "kind": "footprint",
      "name": "DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY"
    }
  ]
}
```

各字段含义：
- `severity`：问题严重程度；
- `code`：稳定的机器可读诊断码；
- `message`：面向人的可读描述；
- `object`：主要问题对象；
- `related`：关联的其他对象。

VSCode 等前端工具应基于 `object` 与 `related` 字段做问题定位，而非解析自然语言文本。以上述诊断为例，可直接定位到位号为 `J1` 的器件，并关联到对应封装。

### 8.4 常见诊断码
- `net.too_few_connections`：网络端点少于 2 个；
- `net.unknown_pin`：网络引用了不存在的逻辑引脚；
- `net.unknown_module`：网络引用了不存在的器件；
- `net.electrical_class_mismatch`：电气类型不匹配；
- `pin_pad_map.missing_pad`：逻辑引脚映射到了不存在的物理焊盘；
- `pin_pad_map.uncovered_footprint_pad`：封装存在未被逻辑引脚覆盖的焊盘；
- `net.physical_pad_short`：同一个物理焊盘被多个网络占用；
- `production.unverified_footprint`：生产检查中存在未验证的封装；
- `production.missing_source`：缺少 MPN 或供应商料号。

### 本章小结
1. 原型检查用于校验电路模型的结构自洽性。
2. 生产检查用于校验设计是否接近可投产状态。
3. 诊断信息必须具备机器可读性。
4. 前端 UI 应基于诊断对象定位，而非解析自然语言。

## 第 9 章 从零新增一个示例工程
若要新增一个示例工程（例如 LED 闪烁演示），可遵循以下标准流程：
1. 在 `via-examples/src/` 目录下新建 `led_demo.rs` 文件，按规范编写器件与连线，末尾补充单元测试；
2. 在 `via-examples/src/lib.rs` 中添加 `pub mod led_demo;` 注册模块；
3. 若需通过 CLI 调用（如 `cargo run ... --example led-demo`），需在 `via-cli/src/main.rs` 中添加路由常量与匹配分支。

### 9.1 新建示例文件
文件路径：
```text
C:\Coding\谐波赤道仪\tools\electronics\via-rs\crates\via-examples\src\led_demo.rs
```

完整示例代码如下：
```rust
use via::prelude::*;

#[derive(Debug, Clone)]
pub struct Led {
    id: ModuleId,
}

impl Led {
    pub fn anode(&self) -> PinRef {
        self.id.pin("A")
    }

    pub fn cathode(&self) -> PinRef {
        self.id.pin("K")
    }
}

pub fn led_0805(refdes: &str, value: &str) -> impl Component<Output = Led> {
    part(refdes, value)
        .footprint("LED_0805_2012Metric")
        .pin(pin("A").passive().pad("1"))
        .pin(pin("K").passive().pad("2"))
        .production_note("Bind exact LED color/current/LCSC part before production")
        .verify()
        .handle(|id| Led { id })
}

pub fn led_demo_board() -> Result<Board> {
    let mut d = Design::new("led_demo")
        .rules(Rules::new())
        .units(Unit::Mm);

    let v3v3 = d.power("3V3", Voltage::dc(3.3));
    let led_drive = d.net("LED_DRIVE");
    let gnd = d.ground("GND");

    let input = d.add(
        part("J1", "3V3 input")
            .footprint("PinHeader_1x02_P2.54")
            .pin(pin("3V3").power("3V3").pad("1"))
            .pin(pin("GND").ground().pad("2"))
            .verify(),
    )?;

    let r1 = d.add(parts::resistor_0805("R1", "1k"))?;
    let d1 = d.add(led_0805("D1", "red LED VERIFY"))?;

    v3v3.connect_all(&mut d, [input.pin("3V3"), r1.pin1()]);
    led_drive.connect_all(&mut d, [r1.pin2(), d1.anode()]);
    gnd.connect_all(&mut d, [input.pin("GND"), d1.cathode()]);

    d.check(CheckProfile::Prototype)?;
    d.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn led_demo_is_valid() {
        let board = led_demo_board().unwrap();
        assert_eq!(board.name(), "led_demo");
        assert_eq!(board.modules().count(), 3);
        assert_eq!(board.nets().count(), 3);
    }
}
```

示例虽然简单，但覆盖了完整电路板的核心要素：自定义类型化器件、通用器件、电源网络、普通信号网络、接地网络、原型检查与单元测试。

### 9.2 注册到示例库
编辑文件：
```text
C:\Coding\谐波赤道仪\tools\electronics\via-rs\crates\via-examples\src\lib.rs
```
添加模块声明：
```rust
pub mod led_demo;
```

运行单元测试验证：
```powershell
cargo test -p via-examples
```

### 9.3 注册到 CLI
当前 CLI 默认仅支持 `polar-adjuster` 示例。若要让以下命令生效：
```powershell
cargo run -p via-cli -- check --example led-demo --json
```

需要编辑文件：
```text
C:\Coding\谐波赤道仪\tools\electronics\via-rs\crates\via-cli\src\main.rs
```
添加常量：
```rust
const LED_DEMO: &str = "led-demo";
```
并在 `load_example` 函数中添加匹配分支：
```rust
LED_DEMO => via_examples::led_demo::led_demo_board(),
```

完成这一步后，CLI 即可将命令行中的 `led-demo` 字符串映射到对应的 Rust 函数。

## 第 10 章 导出 KiCad、LCEDA Pro 与快照文件
`via-rs` 严格遵循单一数据源原则，所有导出格式均基于同一份 `Board` 模型生成。

### 10.1 KiCad 导出
```powershell
cargo run -p via-cli -- export --example polar-adjuster --out ..\..\..\electronics\generated\via\polar_adjuster_v0
```

主要输出文件包括：
```text
polar_adjuster_v0.kicad_pro
polar_adjuster_v0.kicad_sch
polar_adjuster_v0.kicad_sym
polar_adjuster_v0.net
via_generated.pretty/
```
该命令生成可审查的 KiCad 工程，作为后续布线的基础底稿，并非最终完成的 PCB 文件。

### 10.2 LCEDA Pro 导出
```powershell
cargo run -p via-cli -- export-lceda-pro --example polar-adjuster --out ..\..\..\electronics\generated\lceda_pro\polar_adjuster_v0.epro2
```

LCEDA Pro 导出会生成 `.epro2` 工程包文件。需要注意的是，`.epro2` 是封闭的二进制包格式，不适合作为长期维护的源文件。`via-rs` 仅负责生成结构正确的导入草案，不应将其作为电路的唯一事实来源。

若导入 LCEDA Pro 出现引脚与焊盘不匹配的问题，应优先校验 Rust 侧的器件模型与封装焊盘事实，而非直接排查 `.epro2` 的内部格式。

### 10.3 快照（Snapshot）导出
```powershell
cargo run -p via-cli -- snapshot --example polar-adjuster --out snapshot.json
```

快照是专门提供给 VSCode 插件及第三方工具的 JSON 格式数据，包含模块、网络、封装几何、设计规则、诊断信息、生产诊断、源签名与哈希值等完整内容。

前端插件只需读取快照即可，无需反向解析 KiCad 文件来重建电路模型。

### 10.4 PCB 草案导出
```powershell
cargo run -p via-cli -- export-pcb --example polar-adjuster --layout layout.json --out board.kicad_pcb
```
该命令会基于 VIA 布局 JSON 中的器件位置、走线、过孔与板框信息，导出 KiCad PCB 草案文件。它并非 Gerber 生成工具。

合理的定位分工是：
```text
via-rs 生成结构正确的设计草案
KiCad/LCEDA 完成最终编辑与 DRC 校验
Gerber 文件由成熟 EDA 工具输出
```

### 10.5 各类导出的关系
所有导出格式并非三套独立逻辑，而是同一份模型的不同形态：
```text
Board
  -> KiCad 原理图/工程
  -> LCEDA Pro 工程包
  -> 快照 JSON
  -> KiCad PCB 草案
```

如果某一导出器需要单独维护一套引脚映射关系，就属于架构层面的风险。所有引脚映射都应统一来源于器件模型。

## 第 11 章 生产就绪度
硬件设计中存在一个常见误区：只要文件能在 EDA 软件中正常打开，就可以直接投板生产。

文件可正常打开仅说明格式合法，不代表物理设计完全正确；而投板生产要求物理层面零错误。

设计能通过原型检查，仅说明电路逻辑自洽，可以进入评审预览阶段；要通过生产检查，还必须满足以下条件：
- 代码中不再有任何 `VERIFY` 标记，或所有待验证项均已通过实物测量确认；
- 所有器件均已填写准确的厂家型号（MPN）或供应商料号；
- KiCad/LCEDA 的 ERC/DRC 校验全部通过；
- 最终 Gerber 文件已完成人工核对。

以当前的 `polar-adjuster` 为例，仍有大量器件料号未最终确定，远未达到投产标准。因此生产检查报错并非冗余提示，而是明确标识该设计仍处于可审查的草案阶段，并非最终生产包。

### 11.1 原型就绪度
原型就绪度代表模型结构基本可靠，至少满足以下要求：
- 所有器件定义完整；
- 所有引脚定义有效；
- 所有网络具备合理的连接端点；
- 电气类型无明显冲突；
- 现有封装事实下，引脚-焊盘映射自洽；
- 无明显的物理焊盘短路问题。

达到原型就绪度即可支持设计评审、预览与草案导出。

### 11.2 生产就绪度
生产就绪度的要求更高，除电路逻辑成立外，还要求生产信息足够完整，至少包括：
- 封装尺寸经过实物测量确认；
- 器件采购来源明确；
- 已填写 MPN 或供应商料号；
- 关键模块引脚定义经过实物验证；
- EDA 工具 DRC 校验通过；
- Gerber 文件经过可视化检查。

`via-rs` 不会直接判定设计是否可以投板，而是会明确列出所有待补齐的生产信息项。

## 第 12 章 常见错误与排查路径
遇到诊断报错时，可先根据诊断码分类，再顺着对象引用回溯到对应的模型定义进行排查。

### 12.1 `net.too_few_connections`
- **含义**：一个网络的连接端点少于 2 个。
- **常见原因**：遗漏了另一端的连接；创建了引脚句柄但未实际使用；测试用例中存在预期的悬空网络。
- **处理方式**：补齐对应连接；若为测试场景下的预期未连接，需在代码中显式声明。不要为了消除报错而无条件忽略该诊断。

### 12.2 `net.unknown_pin`
- **含义**：网络引用了不存在的逻辑引脚。
- **常见原因**：引脚名拼写错误；修改了器件底层的逻辑引脚命名，但连线代码未同步更新。

### 12.3 `pin_pad_map.missing_pad`
- **含义**：逻辑引脚映射到了不存在的物理焊盘。
- **常见原因**：焊盘命名不匹配（例如代码写 `1`，实际封装为 `A1`）；KiCad 与 LCEDA 封装库的引脚编号规则不一致。

### 12.4 `pin_pad_map.uncovered_footprint_pad`
- **含义**：封装上存在未被任何逻辑引脚覆盖的物理焊盘。
- **常见原因**：遗漏了空引脚（NC）定义；模块存在多个接地焊盘但仅定义了一个；封装库更新后新增了散热焊盘，器件模型未同步。

### 12.5 `net.electrical_class_mismatch`
- **含义**：电气类型冲突，例如将 `power:12V` 连接到 `logic:3V3` 的引脚上。
- **常见原因**：接错了网络；将电阻、电容等被动器件错误标记为 `logic` 而非 `passive`。

### 12.6 LCEDA 引脚/焊盘错误的排查顺序
如果 LCEDA Pro 报出引脚与焊盘不匹配的错误，请按以下顺序排查：
1. 定位对应器件的构造函数；
2. 检查逻辑引脚是否正确声明了物理焊盘映射；
3. 核对封装事实中是否包含对应的焊盘编号；
4. 运行 Rust 侧的检查命令，确认模型本身是否自洽；
5. 若 Rust 模型无问题，再排查 LCEDA 导出器逻辑。

排查顺序至关重要：优先修正模型定义，再排查导出器逻辑。切勿将导出器作为修补模型错误的地方。

## 第 13 章 完整工作流与 crate 分层
前面的章节已经分别讲了器件、网络、封装、检查和导出。现在需要把它们连成一条完整工作流。否则你可能知道每个 API 是什么，却不知道在真实项目中应该先做哪一步、后做哪一步。

### 13.1 一条完整的 VIA 工作流
一个典型 VIA 项目的工作流应当是：

```text
1. 在 Rust 中定义电路事实
   - Part
   - pin
   - pin -> pad mapping
   - electrical class
   - net

2. 准备封装事实与封装几何
   - FootprintPads
   - FootprintIr
   - GeneratedFootprint

3. 将重复电路提炼为 Pattern
   - TMC2209 轴
   - DC 输入 + buck 模块
   - 开关输入

4. 运行检查
   - prototype check
   - production check
   - unit tests

5. 导出机器可读快照
   - snapshot JSON
   - diagnostics
   - footprint geometry

6. 用 VSCode 插件预览
   - 查看器件和网络
   - 检查封装几何
   - 预览 PCB 草案

7. 导出 EDA 草案
   - KiCad schematic
   - KiCad PCB draft
   - LCEDA Pro package

8. 在成熟 EDA 中做最终制造检查
   - ERC
   - DRC
   - Gerber
   - Gerber viewer
```

这条链路里，`via-rs` 的定位不是取代所有 EDA 软件，而是把“电路事实”和“可检查模型”提前建立起来。这样 KiCad 或 LCEDA 处理的是已经较为干净的工程，而不是一堆引脚、封装和网络都没对齐的半成品。

### 13.2 snapshot 与 layout 的区别
这里尤其要区分两个文件：

```text
snapshot JSON
layout JSON
```

snapshot JSON 来自 Rust，表达的是设计事实：

- 有哪些 modules；
- 有哪些 nets；
- 每个 module 引用了什么 footprint；
- footprint geometry 是什么；
- 当前 diagnostics 是什么；
- 当前规则与源签名是什么。

layout JSON 来自编辑器，表达的是布局状态：

- 器件摆在哪里；
- 旋转多少度；
- 是否锁定；
- 板框在哪里；
- 已有走线 segment 是什么；
- 过孔在哪里；
- 哪些对象已经失效。

所以：

```text
Rust 修改电路 -> 重新生成 snapshot
编辑器摆件布线 -> 保存 layout
导出 PCB -> snapshot + layout 合并生成 .kicad_pcb
```

不要把 layout 当成电路事实来源。layout 只保存“这个事实在 PCB 上怎么摆、怎么走线”。如果 Rust 里删掉了某个器件，layout 中对应的 placement 应该变成失效对象，而不是偷偷继续当作有效电路。

### 13.3 crate 分层
当前 workspace 的 crate 分层大致如下：

| crate | 职责 |
| --- | --- |
| `via` | 用户入口 facade，提供 `via::prelude::*`、`via::parts`、`via::patterns` |
| `via-core` | 核心电路模型：`Design`、`Board`、`Part`、`Pin`、`Net`、`FootprintPads`、diagnostics |
| `via-parts` | 通用器件库，例如电阻、电容 |
| `via-parts-harmonic` | 本项目复用器件，例如 ESP32-S3、TMC2209、DC005、MP1584 |
| `via-patterns-motion` | 运动控制相关 pattern，例如 TMC2209 UART 轴 |
| `via-patterns-harmonic` | 本项目 pattern，例如 DC 输入 buck、电平开关输入 |
| `via-footprint-ir` | 底层 PCB footprint 几何模型 |
| `via-footprint` | 高层 footprint generator 与 metadata |
| `via-kicad` | KiCad 导出与 KiCad footprint 解析 |
| `via-lceda-pro` | LCEDA Pro `.epro2` 导出 |
| `via-examples` | 示例电路板与项目电路 |
| `via-cli` | 命令行 check/export/snapshot 包装 |

日常写电路时，优先从 `via` crate 进入：

```rust
use via::prelude::*;
use via::parts;
use via::patterns;
```

只有在写封装生成器、导出器或底层检查逻辑时，才需要直接进入 `via-core`、`via-footprint-ir`、`via-kicad` 等内部 crate。

### 13.4 用户应该在哪里写自己的板子
如果你是在项目内部做 example，放在：

```text
tools/electronics/via-rs/crates/via-examples/src/
```

如果你是在做一个可复用器件，放在：

```text
tools/electronics/via-rs/crates/via-parts-harmonic/src/
```

如果你是在做一组可复用电路块，放在：

```text
tools/electronics/via-rs/crates/via-patterns-harmonic/src/
tools/electronics/via-rs/crates/via-patterns-motion/src/
```

如果你是在做新的 footprint generator，通常放在：

```text
tools/electronics/via-rs/crates/via-footprint/src/generators/
```

如果某个东西只属于 `polar_adjuster_v0`，先放在 `via-examples` 里；如果后来发现第二块板也需要它，再上提到 `via-parts-harmonic` 或 `via-patterns-harmonic`。不要一开始就把所有临时逻辑塞进公共库。

### 13.5 本章小结
VIA 的正确使用方式可以概括为：

```text
核心事实在 Rust
物理几何在 FootprintIr
布局状态在 layout JSON
EDA 文件只是导出结果
```

只要守住这个边界，后续无论导出 KiCad、LCEDA，还是做 VSCode 预览，都不会出现三套模型互相打架的情况。

## 第 14 章 Component 生命周期与 typed API
第 5 章已经讲了 typed part 的基本写法，但还没有讲清 `Component` 的生命周期。本章补上这一层，因为它决定了这个库能不能保持优雅。

### 14.1 `part(...)` 到底返回什么
当你写：

```rust
part("R1", "1k")
    .footprint("R_0805_2012Metric")
    .pin(pin("1").passive().pad("1"))
    .pin(pin("2").passive().pad("2"))
```

此时它还没有进入电路板。它只是一个 `PartSpecBuilder`，可以理解为“准备加入设计的器件描述”。

真正加入设计的是：

```rust
let r1 = design.add(parts::resistor_0805("R1", "1k"))?;
```

`design.add(...)` 会做三件事：

1. 取出器件定义；
2. 加入 `BoardSpec`；
3. 返回这个组件的 output。

对于普通未类型化器件，output 可能只是 `ModuleId` 或 `NetlessPartHandle`。对于 typed part，output 是你自己定义的 handle。

### 14.2 `.handle(...)` 的意义
考虑一个传感器：

```rust
#[derive(Debug, Clone)]
pub struct Sensor {
    id: ModuleId,
}

impl Sensor {
    pub fn vcc(&self) -> PinRef { self.id.pin("VCC") }
    pub fn gnd(&self) -> PinRef { self.id.pin("GND") }
    pub fn scl(&self) -> PinRef { self.id.pin("SCL") }
    pub fn sda(&self) -> PinRef { self.id.pin("SDA") }
}

pub fn sensor(refdes: &str) -> impl Component<Output = Sensor> {
    part(refdes, "I2C sensor module")
        .footprint("Sensor_1x04")
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .handle(|id| Sensor { id })
}
```

`.handle(|id| Sensor { id })` 的作用是：在器件被加入设计后，用实际的 `ModuleId` 构造一个 typed handle。这个 handle 不保存网络，也不保存焊盘几何；它只是安全地提供 pin 引用方法。

因此用户可以写：

```rust
let sensor = design.add(sensor("U5"))?;

v3v3.connect(&mut design, sensor.vcc());
gnd.connect(&mut design, sensor.gnd());
i2c_scl.connect(&mut design, sensor.scl());
i2c_sda.connect(&mut design, sensor.sda());
```

这比 `sensor.pin("SCL")` 更好，因为 IDE 能补全，重构时也更容易定位。

### 14.3 `Component` 不只适用于单个器件
`Component` 的关键价值在于：它既可以表示一个器件，也可以表示一组器件。

例如：

```rust
let x_axis = design.add(
    patterns::Tmc2209UartAxisSpec::new("X")
        .driver("U2")
        .motor_connector("J2")
        .uart_resistor("R1")
        .pins(patterns::Tmc2209UartAxisPins::new(
            esp32.gpio4(),
            esp32.gpio5(),
            esp32.gpio6(),
            esp32.gpio7(),
            esp32.gpio15(),
        )),
)?;
```

这里 `Tmc2209UartAxisSpec` 不是单个器件。它内部会加入 TMC2209 模块、电机连接器和 UART 串联电阻，还会建立若干网络。最后返回 `Tmc2209UartAxis`，让用户继续连接 `vmot()`、`vio()`、`ground()` 等关键 pin。

这就是 VIA 抽象的核心：

```text
小组件是 Part
大组件是 Pattern
二者都实现 Component
design.add(...) 统一接收它们
```

### 14.4 typed API 的边界
typed handle 不应该变成数据手册的完整复制品。比如 ESP32 模块可能有大量 GPIO，但当前板子只使用其中一部分。当前项目中的 `Esp32S3N16R8` 就只暴露实际用到的 pin：

```rust
esp32.gpio4()
esp32.gpio5()
esp32.gpio6()
esp32.gpio7()
esp32.gpio9()
esp32.gpio10()
esp32.power_3v3()
esp32.power_5v()
esp32.ground()
```

如果以后要用新的 GPIO，再补对应方法。不要为了“看起来完整”一次性塞入几十个暂时不用的方法。API 越大，维护负担越大，错误入口也越多。

### 14.5 未用引脚与 NC
复杂模块常有未使用引脚。处理策略要分清两件事：

1. 这个 physical pad 是否存在；
2. 这个 logical pin 是否参与电路网络。

如果封装上有一个 pad 是 NC，可以在 part 中显式建模：

```rust
.pin(pin("NC_J1_3").pad("3"))
```

但不要把它接入任何 net。这样做的好处是：

- `FootprintPads` 不会报 uncovered pad；
- 原理图/导出器知道这个焊盘存在；
- 电气连接里不会凭空多出一根线。

当前 VIA 尚未把 KiCad 的 no-connect marker 做成完整公开 API，因此在 Rust 模型中，“存在但不连接”的 pin 是最直接的表达方式。后续如果补正式 no-connect 语义，应当仍然以 `Part` 为事实源，而不是在导出器里临时补。

## 第 15 章 规则、线宽、间距与过孔
PCB 不是只要网络连上就行。不同网络需要不同线宽，过孔也必须有合理孔径和外径。VIA 的 `BoardRules` 用来描述这些基础约束。

### 15.1 默认规则
默认规则大致如下：

| 规则 | 默认值 |
| --- | --- |
| grid | `2.0 mm` |
| default track width | `0.3 mm` |
| clearance | `0.2 mm` |
| via diameter | `0.8 mm` |
| via drill | `0.4 mm` |

同时内置了一组按电气类型区分的线宽：

| net class | 线宽 |
| --- | --- |
| `ground` | `0.6 mm` |
| `power:12V` | `0.8 mm` |
| `power:5V` | `0.6 mm` |
| `power:3V3` | `0.5 mm` |
| `logic:3V3` | `0.25 mm` |
| `motor-phase` | `0.5 mm` |

这些值不是制造承诺，只是当前项目的合理初始值。最终仍需按板厂能力、电流、温升和布局空间调整。

### 15.2 修改全局规则
可以在创建设计时传入规则：

```rust
let mut rules = Rules::new();
rules
    .set_grid_mm(1.0)
    .set_default_track_width_mm(0.25)
    .set_clearance_mm(0.2)
    .set_via(0.8, 0.4);

let mut design = Design::new("my_board")
    .rules(rules)
    .units(Unit::Mm);
```

也可以在设计创建后修改：

```rust
design.rules_mut().set_clearance_mm(0.25);
```

这适合在 example 中根据板子尺寸和工艺能力快速调整。

### 15.3 修改某类网络线宽
如果需要让 12V 电源更粗：

```rust
design
    .rules_mut()
    .set_net_class_track_width_mm("power:12V", 1.0);
```

如果需要让电机相线更粗：

```rust
design
    .rules_mut()
    .set_net_class_track_width_mm("motor-phase", 0.8);
```

这里的 class 名来自 `ElectricalClass` 的字符串表示：

```text
ground
power:12V
power:5V
logic:3V3
motor-phase
```

因此，如果你创建了 `power_domain("VBAT", "battery")`，它的 class 就会类似 `power:battery`。对应线宽也应使用同一个字符串。

### 15.4 规则如何进入编辑器和导出器
规则属于 `Board` 的一部分。snapshot 导出时应包含规则，VSCode PCB editor 应读取它来决定默认线宽、过孔尺寸和 DRC 阈值。

正确的链路是：

```text
BoardRules
  -> Board
  -> snapshot JSON
  -> VSCode editor default route width / via size
  -> export-pcb
```

如果 UI 中线宽控件显示 `0.30 mm`，它应该来自规则或当前 route session，而不是硬编码在前端。

### 15.5 VIA 当前规则的边界
当前 `BoardRules` 主要描述基础制造规则，尚未完整覆盖：

- 差分对；
- 阻抗控制；
- 区域铺铜规则；
- 多层叠层；
- 热焊盘策略；
- net class 继承；
- 特定封装局部规则。

这些属于后续扩展。第一版最重要的是把线宽、间距和过孔从“写死在 UI 里”推进到“来自 Rust board model 的规则”。

## 第 16 章 电源 Rail 与去耦电容
电源网络比普通信号网络更特殊，因为它经常需要连接一批负载，还要就近放置去耦电容。VIA 提供了 `NetHandle::decouple(...)`，让这种写法更清楚。

### 16.1 普通连接写法
最直接的写法是：

```rust
let v3v3 = design.power("3V3", Voltage::dc(3.3));
let gnd = design.ground("GND");

let esp32 = design.add(parts::esp32_s3_n16r8("U1"))?;
let c4 = design.add(parts::capacitor_0805("C4", "100nF 50V VIO decoupling"))?;

v3v3.connect_all(&mut design, [esp32.power_3v3(), c4.pin1()]);
gnd.connect(&mut design, c4.pin2());
```

这当然可行，但去耦电容写多了会重复。

### 16.2 使用 `decouple`
更清晰的写法是：

```rust
let v3v3 = design.power("3V3", Voltage::dc(3.3));
let c4 = design.add(parts::capacitor_0805("C4", "100nF 50V VIO decoupling"))?;

v3v3.decouple(&mut design, &c4);
```

`decouple` 做了两件事：

1. 将电容正端或 `pin1()` 接到当前电源 rail；
2. 将电容负端或 `pin2()` 接到默认 `GND`。

当前 `Capacitor2` 实现了 `DecouplerPins`，所以可以直接传 `&c4`。

### 16.3 使用 `decouple_to`
如果你的地不是默认 `GND`，可以指定地网络：

```rust
v3v3.decouple_to(&mut design, "AGND", &analog_decoupler);
```

这适合模拟电源、隔离电源或噪声敏感模块。不要为了省事把所有地都强行叫 `GND`，地的命名也是设计意图的一部分。

### 16.4 bulk capacitor 与 decoupling capacitor
常见电容可以大致分为几类：

| 类型 | 常见容量 | 作用 |
| --- | --- | --- |
| 高频去耦 | `100nF` | 给芯片局部瞬态电流，抑制高频噪声 |
| 局部储能 | `1uF` 到 `10uF` | 给模块局部供电缓冲 |
| 输入 bulk | `47uF` 到 `220uF` | 缓冲外部输入电源波动 |
| 电机电源 bulk | `100uF` 以上 | 缓冲驱动器 VMOT 电流尖峰 |

在 `polar_adjuster` 里：

```rust
v12.decouple(&mut design, &x_vmot_bulk)
    .decouple(&mut design, &y_vmot_bulk);

v5.decouple(&mut design, &esp32_5v_local);

v3v3
    .connect_all(&mut design, [esp32.power_3v3(), x_axis.vio(), y_axis.vio()])
    .decouple(&mut design, &x_vio_decouple)
    .decouple(&mut design, &y_vio_decouple);
```

这段代码的含义是：

- `C3/C5` 是 X/Y 驱动器 VMOT bulk；
- `C10` 是 ESP32 5V 本地电容；
- `C4/C6` 是 X/Y TMC2209 VIO 的 3.3V 去耦。

### 16.5 去耦 API 不等于物理位置正确
`decouple(...)` 只表达电气连接，不表达电容应该摆在哪里。真正的物理要求是：

- 高频去耦应靠近对应芯片电源脚；
- VMOT bulk 应靠近驱动器 VMOT/GND；
- 输入 bulk 应靠近电源入口；
- 回流路径要短；
- 大电流回路不要穿过敏感逻辑地。

这些属于 PCB layout 层面的规则。未来 VSCode editor 可以通过 pattern 输出 placement hint，但当前模型里，`decouple` 只负责电气事实，不负责摆件。

## 第 17 章 封装生成器实战
前面已经讲过 `FootprintIr` 的概念，现在补一个更工程化的流程：如何写一个可维护的 footprint generator。

### 17.1 一个 generator 应该输出什么
一个合格的 footprint generator 不应只输出 `.kicad_mod` 文本，而应输出 `GeneratedFootprint`：

```text
GeneratedFootprint
  - FootprintIr
  - FootprintMetadata
```

`FootprintIr` 是几何，`FootprintMetadata` 是来源和验证状态。二者结合后，系统才能同时支持：

- 写 `.kicad_mod`；
- 导出 snapshot 几何；
- 抽取 `FootprintPads`；
- 记录这个 footprint 是否仍需实物验证。

### 17.2 最小 SMD 两端封装
一个简化的 SMD 两端器件 generator 可以写成：

```rust
use via_footprint::{FootprintMetadata, FootprintVerificationStatus, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicLine, GraphicText, Pad, PadShape, Point, Size};

pub fn demo_0805(name: impl Into<String>) -> GeneratedFootprint {
    let name = name.into();
    let mut fp = FootprintIr::new(name.clone())
        .description("Demo 0805 footprint")
        .tag("via-generated")
        .tag("0805");

    fp.add_pad(Pad::smd(
        "1",
        PadShape::Rect,
        Point::new(-0.95, 0.0),
        Size::new(1.15, 1.35),
    ));
    fp.add_pad(Pad::smd(
        "2",
        PadShape::Rect,
        Point::new(0.95, 0.0),
        Size::new(1.15, 1.35),
    ));

    fp.add_line(GraphicLine::new(
        Point::new(-1.0, -0.65),
        Point::new(1.0, -0.65),
        "F.Fab",
        0.08,
    ));

    fp.add_text(GraphicText::reference("REF**", Point::new(0.0, -1.2), "F.SilkS"));
    fp.add_text(GraphicText::value(name.as_str(), Point::new(0.0, 1.2), "F.Fab"));

    GeneratedFootprint::new(fp, FootprintMetadata::generated("demo_0805"))
}
```

真实 generator 通常会把矩形外框、reference/value 文本、courtyard 等公共逻辑提取到 helper 中，避免每个封装都手写一遍。

### 17.3 metadata 与 VERIFY
默认：

```rust
FootprintMetadata::generated("demo_0805")
```

会把验证状态设为 `VerifyRequired`。这意味着生产检查会认为它仍需实物确认。

如果某个 footprint 已经通过实物测量，可以显式标记：

```rust
FootprintMetadata::generated("demo_0805")
    .with_verification_status(FootprintVerificationStatus::Verified)
    .notes("Measured against purchased part on 2026-07-05")
```

不要随便把 footprint 标成 verified。verified 的意思不是“看起来像”，而是：

- 数据手册与实物一致；
- 引脚编号经确认；
- 焊盘坐标与尺寸经核对；
- 关键孔径、槽孔、外形尺寸经检查；
- 导出的 KiCad/LCEDA footprint 能被 EDA 正确识别。

### 17.4 从 geometry 得到 pad facts
一旦有了 `FootprintIr`，就不应该再手写一份 pad facts。正确做法是：

```rust
let pads = FootprintPads::from_ir(footprint.clone().into_ir());
```

项目中通常统一由：

```rust
pub fn generated_footprint_pads() -> Vec<FootprintPads> {
    generated_footprints()
        .into_iter()
        .map(|footprint| FootprintPads::from_ir(footprint.into_ir()))
        .collect()
}
```

这就保证了：

```text
PCB geometry 里的 pad
core checker 看到的 pad
导出器看到的 pad
```

来自同一份事实。

### 17.5 DC005 这类特殊封装
特殊封装要特别谨慎。以 DC005 为例，不能只写：

```rust
Pad::thru_hole("4", PadShape::Circle, Point::new(...), Size::new(...), 1.0)
```

如果实物图纸显示它是长圆孔，就应写：

```rust
Pad::thru_hole_slot(
    "4",
    PadShape::Oval,
    Point::new(7.5, 0.0),
    Size::new(3.0, 1.8),
    2.2,
    0.9,
)
```

特殊封装的最低验收标准是：

1. pad number 与实物或图纸一致；
2. 通孔、槽孔、非金属孔类型正确；
3. 焊盘形状正确；
4. 外形层能表达器件占板空间；
5. `Reference` 和 `Value` 使用正确 text kind；
6. 不把 `REF**` 或 footprint 名字当作普通 user text 随便画；
7. 导入 KiCad/LCEDA 后不会报引脚与焊盘不对应。

## 第 18 章 测试、CI 与版本演进
VIA 的价值很大一部分来自“可测试”。如果 Rust 电路没有测试，就只是在用代码写网表，优势会大打折扣。

### 18.1 最基础的 example 测试
每个 example 至少应该有一个可构建测试：

```rust
#[test]
fn board_is_valid() {
    let board = polar_adjuster_v0_board().unwrap();
    board.check().unwrap();
}
```

这能保证：

- 所有 module refdes 不重复；
- 所有 net 引用的 pin 存在；
- 电气类型没有明显冲突；
- pin-pad 映射自洽；
- 没有物理焊盘短接。

### 18.2 网络断言
关键网络应该写显式断言。比如：

```rust
fn assert_net(board: &Board, name: &str, expected: &[(&str, &str)]) {
    let net = board
        .nets()
        .find(|net| net.name() == name)
        .unwrap_or_else(|| panic!("missing net {name}"));

    let actual = net
        .connections()
        .iter()
        .map(|pin| (pin.module.as_str(), pin.pin.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(actual, expected, "{name}");
}
```

然后写：

```rust
assert_net(
    &board,
    "X_STEP",
    &[("U1", "GPIO7"), ("U2", "STEP")],
);
```

这种测试比肉眼看原理图可靠得多。尤其是你改 ESP32 引脚分配时，它能立刻告诉你是否把 X/Y 轴接反。

### 18.3 footprint 测试
封装 generator 也应该测试。例如：

```rust
#[test]
fn dc005_has_expected_pad_count() {
    let fp = generated_footprints()
        .into_iter()
        .find(|fp| fp.name().contains("DC005"))
        .unwrap();

    let ir = fp.into_ir();
    assert_eq!(ir.pads().len(), 3);
}
```

更严格的测试应检查：

- pad number 集合；
- pad kind；
- pad shape；
- slot drill；
- outline 是否存在；
- reference/value text kind 是否正确。

这能避免“DC005 明明应该是长圆孔，结果又被改成圆孔”的回归。

### 18.4 production check 测试
生产检查可以作为门禁，但不要一开始就强制所有 example 通过 production。更合理的做法是：

- prototype example 必须通过 `check`；
- production-ready example 才必须通过 `check-production`；
- 带 `VERIFY` 的研究板允许 production check 报错，但要把原因写清楚。

在 CI 中可以分层：

```powershell
cargo test -p via-core -p via-parts -p via-footprint-ir
cargo test -p via-kicad -p via-lceda-pro
cargo test -p via-examples
cargo run -p via-cli -- check --example polar-adjuster --json
```

### 18.5 导出 smoke test
导出器至少要有 smoke test，确认不会生成空文件或明显非法文件。例如：

```powershell
cargo run -p via-cli -- export --example polar-adjuster --out ./tmp/polar_adjuster_v0
cargo run -p via-cli -- snapshot --example polar-adjuster --out ./tmp/snapshot.json
cargo run -p via-cli -- export-lceda-pro --example polar-adjuster --out ./tmp/polar_adjuster_v0.epro2
```

如果本机有 KiCad CLI，还可以进一步做 KiCad 打开/导出检查。没有 KiCad CLI 时，至少检查文件存在、非空，并包含关键 refdes 与 footprint 名称。

### 18.6 版本演进原则
VIA 还在快速演进，API 设计要遵守几个原则：

1. 新功能先在 example 中验证，再上提到公共 crate；
2. 器件事实、symbol 样式、footprint 几何不要混在一个结构里；
3. 导出器不应该补写电路逻辑；
4. snapshot/layout schema 变更要有版本号；
5. 破坏性 schema 变更要写迁移或明确“不兼容重建”；
6. 公共 API 越小越好，但核心概念必须完整；
7. 测试必须覆盖 bug 的根因，而不是只覆盖当前输出文本。

这套原则比“今天能导出一个文件”更重要。文件能导出只是第一步；能长期维护、能被别人扩展，才是库成熟的标志。

---

## 附录 A 当前 API 速查

**初始化：**
```rust
let mut d = Design::new("board_name")
    .rules(Rules::new())
    .units(Unit::Mm);
```

**网络定义：**
```rust
d.net("SIGNAL");
d.logic("UART", "3V3");
d.power("12V_IN", Voltage::dc(12.0));
d.ground("GND");
d.motor_phase("X_OA1");
```

**器件与引脚构建：**
```rust
part("U1", "module value")
    .footprint("Footprint_Name")
    .pin(pin("VCC").power("3V3").pad("1"))
    .pin(pin("GND").ground().pads(["2", "3"])) // 多焊盘映射
    .pin(pin("1").passive()) // 默认同名焊盘
    .verify() // 标记需实物测量验证
    .lcsc("Cxxxx")
    .handle(|id| MyHandle { id })
```

## 附录 B 常用命令
```powershell
# 运行单元测试
cargo test

# 日常原型检查（加 --json 可输出机器可读格式）
cargo run -p via-cli -- check --example polar-adjuster

# 生产级门禁检查
cargo run -p via-cli -- check-production --example polar-adjuster

# 导出供插件使用的快照文件
cargo run -p via-cli -- snapshot --example polar-adjuster --out snapshot.json

# 导出各类 EDA 文件
cargo run -p via-cli -- export --example polar-adjuster --out ./via/polar_adjuster_v0
cargo run -p via-cli -- export-lceda-pro --example polar-adjuster --out ./lceda/polar_adjuster_v0.epro2
```
