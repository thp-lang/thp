--TEST--
Throwable subtype catches and finally unwind every source transfer
--FILE--
<?thp

class CustomFailure extends Exception {}

function makeReturnValue(): string {
    echo "return-evaluated\n";
    return "returned";
}

function returnThroughFinally(): string {
    try {
        return makeReturnValue();
    } finally {
        echo "return-finally\n";
    }
}

function replacedReturn(): string {
    try {
        return "body-return";
    } finally {
        return "finally-return";
    }
}

try {
    echo "body\n";
} finally {
    echo "normal-finally\n";
}

echo returnThroughFinally() . "\n";
echo replacedReturn() . "\n";

$index: int = 0;
while ($index < 3) {
    $index = $index + 1;
    try {
        if ($index === 1) {
            continue;
        }
        if ($index === 2) {
            break;
        }
    } finally {
        echo "loop-finally:" . $index . "\n";
    }
}

$continueWins: int = 0;
while ($continueWins < 3) {
    try {
        break;
    } finally {
        $continueWins = $continueWins + 1;
        continue;
    }
}
echo "continue-wins:" . $continueWins . "\n";

$breakWins: int = 0;
while ($breakWins < 3) {
    $breakWins = $breakWins + 1;
    try {
        continue;
    } finally {
        break;
    }
}
echo "break-wins:" . $breakWins . "\n";

try {
    throw new CustomFailure("custom", 7);
} catch (CustomFailure $error) {
    echo $error->getMessage() . ":" . $error->getCode() . "\n";
} finally {
    echo "catch-finally\n";
}

try {
    try {
        throw new CustomFailure("unmatched");
    } finally {
        echo "unmatched-finally\n";
    }
} catch (CustomFailure $error) {
    echo $error->getMessage() . "\n";
}

try {
    try {
        throw new Exception("caught-first");
    } catch (Exception $error) {
        throw new CustomFailure("catch-throw");
    } finally {
        echo "catch-throw-finally\n";
    }
} catch (CustomFailure $error) {
    echo $error->getMessage() . "\n";
}

try {
    try {
        echo "nested-body\n";
    } finally {
        echo "nested-inner\n";
    }
} finally {
    echo "nested-outer\n";
}

try {
    try {
        throw new Exception("pending");
    } finally {
        throw new Exception("replacement", 9);
    }
} catch (Exception $error) {
    echo $error->getMessage() . ":" . $error->getCode() . "\n";
    var_dump($error->getPrevious() instanceof Exception);
}
--EXPECT--
body
normal-finally
return-evaluated
return-finally
returned
finally-return
loop-finally:1
loop-finally:2
continue-wins:3
break-wins:1
custom:7
catch-finally
unmatched-finally
unmatched
catch-throw-finally
catch-throw
nested-body
nested-inner
nested-outer
replacement:9
bool(true)
