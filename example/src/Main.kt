fun main() {
    val prefix = Prefix().success()
    val suffix = Suffix().ful()
    println("$prefix$suffix")
}

class Prefix {
    fun success(): String = "success"
}

class Suffix {
    fun ful(): String = "ful"
}
