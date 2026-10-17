package acme

fun testTotal() {
    check(Order(2, 500).total() == 1000)
}
