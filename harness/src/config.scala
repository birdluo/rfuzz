// Copyright 2017 Kevin Laeufer <laeufer@cs.berkeley.edu>
package pynq

import scala.collection.immutable.ListMap
// this file contains code to load our TOML config files and
// extract the information needed to generate the harness

case class DUTConfig(src: String, name: String, input: Seq[(String, Int)],
                     inputBits: Int, coverageSignals: Int,
                     coverageCounters: Seq[(String, Int)])

object Config {
	import java.time.OffsetDateTime
	import java.nio.file.{Paths, Files}
	import java.nio.charset.StandardCharsets
	import toml._
	import toml.Codecs._

	implicit val booleanCodec: toml.Codec[Boolean] = toml.Codec[Boolean]( {
		case toml.Value.Bool(value) => Right(value)
		case value  =>
			Left((List.empty, s"Boolean expected, $value provided"))
	})
	// TODO: why does this not work out of the box? why do we need to reimplement this here?
	implicit val offsetDateTimeCodec: toml.Codec[OffsetDateTime] = toml.Codec[OffsetDateTime]( {
		case toml.Value.OffsetDateTime(value) => Right(value)
		case value  =>
			Left((List.empty, s"OffsetDateTime expected, $value provided"))
	})
	// TODO: this seems like quite a hack .... why does the `tableCodec` from the lib not work?
	implicit val intTableCodec: toml.Codec[Seq[(String,Int)]] = toml.Codec[Seq[(String,Int)]]( {
		case toml.Value.Tbl(values) => {
			Right(values.map{
				case (key: String, num: toml.Value.Num) => (key, num.value.toInt)
				case (k, v) => throw new Exception(s"Unexpected ${k} -> ${v}, expected String -> Int")
			}.toSeq)
		}
		case value  =>
			Left((List.empty, s"Table expected, $value provided"))
	})

	def loadFileContent(filename: String) : String = {
		new String(Files.readAllBytes(Paths.get(filename)), StandardCharsets.UTF_8)
	}
	def loadToml(filename: String) : DUTConfig = {
		case class General(filename: String, module: String, instrumented: String, timestamp: OffsetDateTime)
		case class Coverage(name: String, inverted: Boolean, index: Int, counterbits: Int, filename: String, line: Int, column: Int, human: String)
		case class Test(general: General, coverage: List[Coverage], input: Seq[(String, Int)])
		val Right(toml) = Toml.parseAs[Test](loadFileContent(filename))
		// DEBUG print
		// println(s"Module: ${toml.general.module}")
		// println(s"Created on: ${toml.general.timestamp}")
		// println(s"Inputs:")
		// for((name, width) <- toml.input) {
		// 	println(s"${name}: ${width}")
		// }
		// extract relevant values
		val src = toml.general.instrumented
		val name = toml.general.module
		val input = toml.input
		val inputBits = input.map{ case (_,w) => w }.reduce(_+_)
		val coverageSignals = toml.coverage.size
		val coverageCounters = toml.coverage.map{ case cov: Coverage => (cov.name -> cov.counterbits) }
		new DUTConfig(src, name, input, inputBits, coverageSignals, coverageCounters)
	}
}