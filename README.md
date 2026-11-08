# Rust Developer Basic

Откройте в VS Code корень курса с [Cargo.toml](Cargo.toml). Копируйте команды из блоков комментариев и выполняйте их в PowerShell из корня курса.

Для практики в разделах приведены задачи LeetCode или самостоятельные задания на Rust. Подсказки к задачам LeetCode связывают условия с приёмами из урока.

## Модуль 1. Введение в Rust

### 1.1 Знакомство с языком Rust

- [Знакомство с языком Rust](<Модуль 1/1.1_Знакомство_с_языком_Rust.rs>) — первая программа, ошибки компиляции и выполнения.

- [2235. Add Two Integers](https://leetcode.com/problems/add-two-integers/)
- [2469. Convert the Temperature](https://leetcode.com/problems/convert-the-temperature/)
- [1920. Build Array from Permutation](https://leetcode.com/problems/build-array-from-permutation/)

### 1.2 Подготовка рабочего окружения

- [Подготовка рабочего окружения](<Модуль 1/1.2_Подготовка_рабочего_окружения.rs>) — проверка, сборка, запуск и форматирование одного исходника.

- [Практическое задание. Конвертер минут](<Практические задания/1.2_Конвертер_минут.md>)

### 1.3 Основы архитектуры компьютера

- [Основы архитектуры компьютера](<Модуль 1/1.3_Основы_архитектуры_компьютера.rs>) — процесс, байты в памяти и операции процессора.

Посчитайте и переставьте биты в двоичном представлении числа. В последней задаче смоделируйте сложение с переносом через побитовые операции.

- [191. Number of 1 Bits](https://leetcode.com/problems/number-of-1-bits/)
- [190. Reverse Bits](https://leetcode.com/problems/reverse-bits/)
- [371. Sum of Two Integers](https://leetcode.com/problems/sum-of-two-integers/)

## Модуль 2. Синтаксис Rust и основы Computer Science

### 2.1 Типы данных, переменные и функции

- [Типы данных](<Модуль 2/2.1_Типы_данных.rs>) — скалярные типы, их границы и преобразования.
- [Переменные](<Модуль 2/2.1_Переменные.rs>) — инициализация, изменяемость, константы и затенение.
- [Функции](<Модуль 2/2.1_Функции.rs>) — параметры, аргументы, вызовы и возвращаемые значения.

- [258. Add Digits](https://leetcode.com/problems/add-digits/)
- [1281. Subtract the Product and Sum of Digits of an Integer](https://leetcode.com/problems/subtract-the-product-and-sum-of-digits-of-an-integer/)
- [9. Palindrome Number](https://leetcode.com/problems/palindrome-number/)

### 2.2 Сложные типы данных

- [Кортежи](<Модуль 2/2.2_Кортежи.rs>) — поля по позиции, распаковка, вложенность и возврат связанных значений.
- [Структуры](<Модуль 2/2.2_Структуры.rs>) — именованные поля, обновление экземпляра и кортежные структуры.
- [Перечисления](<Модуль 2/2.2_Перечисления.rs>) — варианты одного типа, данные вариантов и смена варианта.

Представьте координаты кортежами, парковку структурой, а исход игры перечислением.

- [1232. Check If It Is a Straight Line](https://leetcode.com/problems/check-if-it-is-a-straight-line/)
- [1603. Design Parking System](https://leetcode.com/problems/design-parking-system/)
- [1275. Find Winner on a Tic Tac Toe Game](https://leetcode.com/problems/find-winner-on-a-tic-tac-toe-game/)

### 2.3 Ветвление и циклы

- [Ветвление](<Модуль 2/2.3_Ветвление.rs>) — условия, выбор ветки и значение выражения if.
- [Циклы](<Модуль 2/2.3_Циклы.rs>) — диапазоны, условия завершения, результат цикла и метки.

- [412. Fizz Buzz](https://leetcode.com/problems/fizz-buzz/)
- [485. Max Consecutive Ones](https://leetcode.com/problems/max-consecutive-ones/)
- [367. Valid Perfect Square](https://leetcode.com/problems/valid-perfect-square/)

### 2.4 Практика. Типы данных, переменные, функции, ветвление и циклы

- [Прикладные задачи](<Модуль 2/2.4_Практика.rs>) — девять задач с условиями, решениями и проверками.

- [13. Roman to Integer](https://leetcode.com/problems/roman-to-integer/)
- [58. Length of Last Word](https://leetcode.com/problems/length-of-last-word/)
- [121. Best Time to Buy and Sell Stock](https://leetcode.com/problems/best-time-to-buy-and-sell-stock/)

### 2.5 Управление ресурсами, клонирование, Copy и владение

- [Управление ресурсами](<Модуль 2/2.5_Управление_ресурсами.rs>) — открытие файла, освобождение при выходе из блока и досрочное закрытие.
- [Клонирование](<Модуль 2/2.5_Клонирование.rs>) — копия профиля, клонирование поля и восстановление через clone_from.
- [Трейт Copy](<Модуль 2/2.5_Трейт_Copy.rs>) — неявное копирование при присваивании и передаче значения в функцию.
- [Концепция владения](<Модуль 2/2.5_Концепция_владения.rs>) — перемещение значения, передача в функцию, возврат и выход из блока.

Сравните копирование элементов массива с клонированием всей коллекции. В задаче со строкой передайте владение буфером через `into_bytes`, а в списке переставьте узлы без клонирования.

- [1929. Concatenation of Array](https://leetcode.com/problems/concatenation-of-array/)
- [1047. Remove All Adjacent Duplicates In String](https://leetcode.com/problems/remove-all-adjacent-duplicates-in-string/)
- [206. Reverse Linked List](https://leetcode.com/problems/reverse-linked-list/)

### 2.6 Владение и заимствование

- [Владение и заимствование](<Модуль 2/2.6_Владение_и_заимствование.rs>) — чтение через `&`, изменение через `&mut` и передача владения.

Выделите функции, принимающие `&[T]` для чтения и `&mut [T]` для изменений на месте.

- [35. Search Insert Position](https://leetcode.com/problems/search-insert-position/)
- [344. Reverse String](https://leetcode.com/problems/reverse-string/)
- [283. Move Zeroes](https://leetcode.com/problems/move-zeroes/)

### 2.7 Практика владения и заимствования

- [Прикладные задачи](<Модуль 2/2.7_Практика_Владение_и_заимствование.rs>) — восемь задач с условиями, решениями и проверками.

Проверяйте палиндром через заимствованный слайс. При удалении дубликатов и слиянии массивов изменяйте переданный буфер на месте.

- [125. Valid Palindrome](https://leetcode.com/problems/valid-palindrome/)
- [26. Remove Duplicates from Sorted Array](https://leetcode.com/problems/remove-duplicates-from-sorted-array/)
- [88. Merge Sorted Array](https://leetcode.com/problems/merge-sorted-array/)

### 2.8 Коллекции. Слайсы, массивы, строки

- [Слайсы](<Модуль 2/2.8_Слайсы.rs>) — диапазоны, заимствование части буфера, изменение и копирование участка.
- [Массивы](<Модуль 2/2.8_Массивы.rs>) — фиксированная длина, индексы и таблицы из вложенных массивов.
- [Строки](<Модуль 2/2.8_Строки.rs>) — `String`, `&str`, обработка текста и границы UTF-8.

Потренируйтесь обходить слайс чисел, обрабатывать слова через строковые слайсы и искать совпадение в окнах байтов.

- [724. Find Pivot Index](https://leetcode.com/problems/find-pivot-index/)
- [557. Reverse Words in a String III](https://leetcode.com/problems/reverse-words-in-a-string-iii/)
- [28. Find the Index of the First Occurrence in a String](https://leetcode.com/problems/find-the-index-of-the-first-occurrence-in-a-string/)

### 2.9 Структуры и методы. Инкапсуляция

- [Структуры](<Модуль 2/2.9_Структуры.rs>) — модель складской позиции, правила пополнения и результат расчёта.
- [Методы](<Модуль 2/2.9_Методы.rs>) — приёмка, резервирование и отгрузка товара через методы складской позиции.
- [Инкапсуляция](<Модуль 2/2.9_Инкапсуляция.rs>) — закрытое состояние склада и операции, которые сохраняют согласованность остатков.

Спрячьте внутреннее состояние за методами структуры и проверьте его через публичный интерфейс.

- [933. Number of Recent Calls](https://leetcode.com/problems/number-of-recent-calls/)
- [155. Min Stack](https://leetcode.com/problems/min-stack/)
- [1472. Design Browser History](https://leetcode.com/problems/design-browser-history/)

### 2.10 Перечисления и pattern matching

- [Перечисления](<Модуль 2/2.10_Перечисления.rs>) — состояния доставки, сверка номера отправления и данные о выдаче заказа.
- [Pattern matching](<Модуль 2/2.10_Pattern_matching.rs>) — разбор состояний доставки и маршрутизация обращений в очереди поддержки.

Представьте операции, направления и типы скобок вариантами `enum`. Разберите их через `match`.

- [2011. Final Value of Variable After Performing Operations](https://leetcode.com/problems/final-value-of-variable-after-performing-operations/)
- [657. Robot Return to Origin](https://leetcode.com/problems/robot-return-to-origin/)
- [20. Valid Parentheses](https://leetcode.com/problems/valid-parentheses/)

### 2.11 Практика. Структуры, методы, перечисления и pattern matching

- [Практика](<Модуль 2/2.11_Практика_Структуры_и_методы_Перечисления_и_pattern_matching.rs>) — шесть задач на закрытое состояние, методы, перечисления и сопоставление с образцом.

Реализуйте структуры с закрытыми полями. В локальных тестах опишите последовательности вызовов через перечисления команд и выполняйте их через `match`.

- [232. Implement Queue using Stacks](https://leetcode.com/problems/implement-queue-using-stacks/)
- [622. Design Circular Queue](https://leetcode.com/problems/design-circular-queue/)
- [2043. Simple Bank System](https://leetcode.com/problems/simple-bank-system/)

### 2.12 Введение в систему типов. Result, Option, ()

- [Введение в систему типов. Result, Option, ()](<Модуль 2/2.12_Введение_в_систему_типов_Result_Option_().rs>) — учёт складских ячеек, проверка ввода и сброс результатов.

При слиянии строк обработайте `Option` от `next()`. При развороте числа представьте переполнение как `Err` во вспомогательной функции, а на границе с LeetCode верните `0` по условию. В сортировке изменяйте слайс на месте и возвращайте `()`.

- [1768. Merge Strings Alternately](https://leetcode.com/problems/merge-strings-alternately/)
- [7. Reverse Integer](https://leetcode.com/problems/reverse-integer/)
- [75. Sort Colors](https://leetcode.com/problems/sort-colors/)

### 2.13 Введение в лямбда-выражения

Используйте замыкания для преобразования элементов, поиска первого подходящего значения и вычисления ключа сортировки.

- [1431. Kids With the Greatest Number of Candies](https://leetcode.com/problems/kids-with-the-greatest-number-of-candies/)
- [1475. Final Prices With a Special Discount in a Shop](https://leetcode.com/problems/final-prices-with-a-special-discount-in-a-shop/)
- [1859. Sorting the Sentence](https://leetcode.com/problems/sorting-the-sentence/)

### 2.14 Обработка ошибок. Оператор ?, работа с Result и Option

Вынесите чтение цифр, токенов или частей адреса в функции с `Option` и `Result`. Применяйте `?` внутри этих функций и преобразуйте результат в формат ответа LeetCode.

- [8. String to Integer (atoi)](https://leetcode.com/problems/string-to-integer-atoi/)
- [150. Evaluate Reverse Polish Notation](https://leetcode.com/problems/evaluate-reverse-polish-notation/)
- [468. Validate IP Address](https://leetcode.com/problems/validate-ip-address/)

### 2.15 Практика. Работа с Result и Option

При восстановлении адресов используйте `Option` для частей, которые могут оказаться недопустимыми. При проверке байтов и числа возвращайте `Result` с позицией ошибки из вспомогательной функции и преобразуйте его в `bool` для LeetCode.

- [93. Restore IP Addresses](https://leetcode.com/problems/restore-ip-addresses/)
- [393. UTF-8 Validation](https://leetcode.com/problems/utf-8-validation/)
- [65. Valid Number](https://leetcode.com/problems/valid-number/)

## Модуль 3. Типы, полиморфизм и обобщённое программирование

### 3.1 Обобщённое программирование. Статический полиморфизм

Сделайте реализации обобщёнными по типу элемента с подходящими ограничениями трейтов. Сохраните сигнатуры LeetCode как обёртки.

- [704. Binary Search](https://leetcode.com/problems/binary-search/)
- [350. Intersection of Two Arrays II](https://leetcode.com/problems/intersection-of-two-arrays-ii/)
- [215. Kth Largest Element in an Array](https://leetcode.com/problems/kth-largest-element-in-an-array/)

### 3.2 Времена жизни (lifetimes)

- [Практическое задание. Поиск по строкам журнала](<Практические задания/3.2_Поиск_по_строкам_журнала.md>)

### 3.3 Наследование и обобщённое программирование

- [Практическое задание. Общий интерфейс очереди и дека](<Практические задания/3.3_Общий_интерфейс_очереди_и_дека.md>)

### 3.4 Динамический полиморфизм. dyn Trait, dyn Any

- [Практическое задание. Реестр команд](<Практические задания/3.4_Реестр_команд.md>)

### 3.5 Практика. Обобщённое программирование

Передайте функцию угадывания как параметр `FnMut`, обобщите слияние интервалов по типу границ и реализуйте хеш-таблицу с параметрами для ключей и значений.

- [374. Guess Number Higher or Lower](https://leetcode.com/problems/guess-number-higher-or-lower/)
- [56. Merge Intervals](https://leetcode.com/problems/merge-intervals/)
- [706. Design HashMap](https://leetcode.com/problems/design-hashmap/)

### 3.6 Коллекции. Big O, хеш-таблица и вектор

Сравните затраты по времени и памяти у решений на `Vec`, `HashSet` и `HashMap`, учитывая среднюю сложность операций хеш-таблицы.

- [1. Two Sum](https://leetcode.com/problems/two-sum/)
- [217. Contains Duplicate](https://leetcode.com/problems/contains-duplicate/)
- [347. Top K Frequent Elements](https://leetcode.com/problems/top-k-frequent-elements/)

### 3.7 Коллекции и итераторы

Потренируйте `chunks_exact` и `flat_map` при распаковке, `difference` для множеств и `enumerate` при группировке.

- [1313. Decompress Run-Length Encoded List](https://leetcode.com/problems/decompress-run-length-encoded-list/)
- [2215. Find the Difference of Two Arrays](https://leetcode.com/problems/find-the-difference-of-two-arrays/)
- [1282. Group the People Given the Group Size They Belong To](https://leetcode.com/problems/group-the-people-given-the-group-size-they-belong-to/)

### 3.8 Практика. Коллекции и итераторы

Составьте цепочки итераторов для фильтрации слов, вычисления префиксных и суффиксных произведений и отбора частых слов.

- [500. Keyboard Row](https://leetcode.com/problems/keyboard-row/)
- [238. Product of Array Except Self](https://leetcode.com/problems/product-of-array-except-self/)
- [692. Top K Frequent Words](https://leetcode.com/problems/top-k-frequent-words/)

### 3.9 Функции и лямбда-выражения

Передавайте замыкания как ключ сортировки и компаратор. Условие про композицию функций дано для JavaScript, его можно реализовать локально через `Fn` на Rust.

- [973. K Closest Points to Origin](https://leetcode.com/problems/k-closest-points-to-origin/)
- [179. Largest Number](https://leetcode.com/problems/largest-number/)
- [2629. Function Composition](https://leetcode.com/problems/function-composition/)

## Модуль 4. Многопоточность и продвинутые темы

### 4.1 Работа с кучей. Box, Arc, Rc и RAII

Перемещайте узлы списка через `Box`, возвращайте найденное поддерево через `Rc`, а готовый префиксный словарь разделите между читателями через `Arc` в локальном упражнении. Проследите, когда освобождается память.

- [21. Merge Two Sorted Lists](https://leetcode.com/problems/merge-two-sorted-lists/)
- [700. Search in a Binary Search Tree](https://leetcode.com/problems/search-in-a-binary-search-tree/)
- [208. Implement Trie (Prefix Tree)](https://leetcode.com/problems/implement-trie-prefix-tree/)

### 4.2 Потоки. Создание, запуск, Send и Sync

Выполните задачи 4.2–4.5 локально на Rust по условиям LeetCode. В этом пункте запустите отдельные потоки через `thread::spawn`, дождитесь их через `join` и проверьте требования `Send` и `Sync`.

- [1114. Print in Order](https://leetcode.com/problems/print-in-order/)
- [1115. Print FooBar Alternately](https://leetcode.com/problems/print-foobar-alternately/)
- [1116. Print Zero Even Odd](https://leetcode.com/problems/print-zero-even-odd/)

### 4.3 Лямбды и потоки. Использование move и Arc

Переносите замыкания и копии `Arc` в потоки через `move`. Проверяйте порядок вызовов по примерам из условий.

- [1115. Print FooBar Alternately](https://leetcode.com/problems/print-foobar-alternately/)
- [1117. Building H2O](https://leetcode.com/problems/building-h2o/)
- [1195. Fizz Buzz Multithreaded](https://leetcode.com/problems/fizz-buzz-multithreaded/)

### 4.4 Mutex и атомарные типы

Для порядка вывода используйте атомарное состояние, для чередования чисел — `Mutex` с `Condvar`, для философов — мьютексы и справедливый порядок выдачи ресурсов.

- [1114. Print in Order](https://leetcode.com/problems/print-in-order/)
- [1116. Print Zero Even Odd](https://leetcode.com/problems/print-zero-even-odd/)
- [1226. The Dining Philosophers](https://leetcode.com/problems/the-dining-philosophers/)

### 4.5 Практика. Потоки, куча, Mutex и атомарные типы

Напишите локальные интеграционные тесты для координации групп, порядка вывода и захвата ресурсов. Используйте `Arc` для общего состояния, `Mutex` для составных обновлений и атомарные типы для простых счётчиков.

- [1117. Building H2O](https://leetcode.com/problems/building-h2o/)
- [1195. Fizz Buzz Multithreaded](https://leetcode.com/problems/fizz-buzz-multithreaded/)
- [1226. The Dining Philosophers](https://leetcode.com/problems/the-dining-philosophers/)

### 4.6 Внутренняя изменяемость (interior mutability)

Изменяйте узлы дерева через `Rc<RefCell<_>>`. Ограничивайте области `borrow()` и `borrow_mut()` и проследите, когда проверяются заимствования.

- [226. Invert Binary Tree](https://leetcode.com/problems/invert-binary-tree/)
- [701. Insert into a Binary Search Tree](https://leetcode.com/problems/insert-into-a-binary-search-tree/)
- [538. Convert BST to Greater Tree](https://leetcode.com/problems/convert-bst-to-greater-tree/)

### 4.7 Сборка проекта. Организация модулей

- [Практическое задание. Статистика показаний](<Практические задания/4.7_Статистика_показаний.md>)

### 4.8 Продвинутые элементы синтаксиса

Используйте деструктуризацию в проверках судоку, `while let` в обходе дерева и `match` с условиями на границах матрицы.

- [36. Valid Sudoku](https://leetcode.com/problems/valid-sudoku/)
- [102. Binary Tree Level Order Traversal](https://leetcode.com/problems/binary-tree-level-order-traversal/)
- [498. Diagonal Traverse](https://leetcode.com/problems/diagonal-traverse/)

### 4.9 Декларативные и процедурные макросы

- [Практическое задание. Имена полей настроек](<Практические задания/4.9_Имена_полей_настроек.md>)

### 4.10 Асинхронность, цикл событий, проблема 10 тысяч соединений и async/await

- [Практическое задание. Асинхронный TCP-сервер PING/PONG](<Практические задания/4.10_Асинхронный_TCP-сервер_PING-PONG.md>)

### 4.11 Разбор реального приложения на Rust
