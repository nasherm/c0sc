from plox.tool.Expr import *
from plox.plox.TokenType import *
from plox.plox.RuntimeError import *
from plox.plox.Util import runtimeError

class Interpreter(Visitor):
    def __init__(self):
        self.hadRuntimeError = False

    def interpret(self, expr: Expr):
        try:
            value = self.evaluate(expr)
            print(value)
        except RuntimeError as e:
            runtimeError(e)
            self.hadRuntimeError = True

    def visitLiteralExpr(self,expr: Literal):
        return expr.value

    def evaluate(self, expr: Expr):
        return expr.accept(self)

    def visitGroupingExpr(self,expr:Grouping):
        return self.evaluate(expr.expression)

    def checkNumberOperands(self, operator, *operands):
        allFloats = True
        for operand in operands:
            allFloats &= isinstance(operand, float)
        if allFloats:
            return
        raise RuntimeError(operator, 'Operand must be a number')

    def isTruthy(self, obj):
        if isinstance(obj, bool):
            return obj
        elif obj is None:
            return False
        return True

    def visitUnaryExpr(self,expr:Unary):
        right = self.evaluate(expr.right)

        if expr.operator is TokenType.MINUS:
            self.checkNumberOperands(expr.operator, right)
            right = right * -1.0
        elif expr.operator is TokenType.BANG:
            right = not self.isTruthy(right)

        return right

    def visitBinaryExpr(self,expr:Binary):
        left = self.evaluate(expr.left)
        right = self.evaluate(expr.right)
        opType = expr.operator.type

        if opType is TokenType.GREATER:
            self.checkNumberOperands(opType, left, right)
            return left > right
        elif opType is TokenType.GREATER_EQUAL:
            self.checkNumberOperands(opType, left, right)
            return left >= right
        elif opType is TokenType.LESS:
            self.checkNumberOperands(opType, left, right)
            return left < right
        elif opType is TokenType.LESS_EQUAL:
            self.checkNumberOperands(opType, left, right)
            return left <= right
        elif opType is TokenType.MINUS:
            self.checkNumberOperands(opType, left, right)
            return left - right
        elif opType is TokenType.PLUS:
            isTwoNumbers = isinstance(left, float) and isinstance(right, float)
            isTwoStrings = isinstance(left, str) and isinstance(right, str)
            if not (isTwoStrings or isTwoNumbers):
                raise RuntimeError(opType, "Operands must be two strings or two numbers")
            return left + right
        elif opType is TokenType.SLASH:
            self.checkNumberOperands(opType, left, right)
            return left / right
        elif opType is TokenType.STAR:
            self.checkNumberOperands(opType, left, right)
            return left * right
        elif opType is TokenType.BANG_EQUAL:
            return not left == right
        elif opType is TokenType.EQUAL_EQUAL:
            return left == right
