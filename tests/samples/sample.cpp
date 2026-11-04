#include <iostream>
#include <string>
#include <vector>

namespace demo {

template <typename T>
class Stack {
public:
    void push(const T &value) { items_.push_back(value); }
    T pop() {
        T value = items_.back();
        items_.pop_back();
        return value;
    }
    bool empty() const noexcept { return items_.empty(); }

private:
    std::vector<T> items_;
};

} // namespace demo

int main() {
    demo::Stack<std::string> stack;
    stack.push("안녕");
    auto lambda = [](int x) { return x * 2; };
    // 한 줄 주석
    while (!stack.empty()) {
        std::cout << stack.pop() << lambda(21) << std::endl;
    }
    return 0;
}
