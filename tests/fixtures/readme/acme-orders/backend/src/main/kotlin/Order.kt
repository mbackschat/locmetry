package acme

// Prices are represented in cents.
data class Order(val quantity: Int, val unitPrice: Int) {
    fun total(): Int = quantity * unitPrice
}
