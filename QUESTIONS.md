# Questions to CANopen protocols

当我需要使用RP2040来与一些Servo通讯的时候，我发现这样的支持，即使在C上，也非常艰难。慢慢的，我有了个想法，基于python canopen实现另外做一套rust的库，以方便我项目的需要。这个过程比较艰难，初次使用Rust的艰难并不是一个特别巨大的问题，但是对于canopen协议里面的一些困惑，倒成了延缓我脚步的主要障碍。为此，我把实现细节中的一些问题记录在此，确保最终不至于忘记。

## General questions

## SDO questions
SDO是我们目前主要的用法
### Stateless or Stateful?
我原本期望SDO是stateless的，这样不同packet(frame)之间的干扰，可以降到最小。但是在segment / block transfer中，我猜为了包交换的效率，canopen协议放弃了无状态特性。也就是说，当你在segment /block transfer的过程中，任何其它SDO的请求，都会干扰这个传输。

当然，我们期望大多数SDO传输都是对于u32以内数据的读写，这种情况下倒是可以稍微忽略这个问题。只是使用SDO的时候，为了系统稳定性考虑，我们可能需要回避一些超过4 bytes的数据，并坚持用expedite的方式进行数据读写。

### CRC support for client / server?
我在CANopen 301协议中没有看到对于这个的详细描述。所以cc / sc是对client / server端CRC的分别描述？那如果两端的crc能力不一致，是ignore packet呢，报错呢，还是放弃crc verification呢？我没看到清晰的定义。

我目前的实现暂时随便选择了ignore crc verification，不过大家有更好的发现，欢迎发issue让我改正。

### Blocksize for client / server?
不是很理解两端的blksize是怎么协调的。从test cases里面看，server side就是稳定的一个0x7F

### How to check min / max value?
没看懂case SDO.10