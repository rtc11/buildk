import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

internal class PrefixTest {

    @Test
    fun success() {
        val actual = Prefix().success()
        val expected = "success"
        assertEquals(expected, actual)
    }
}
